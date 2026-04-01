//! Entropy Sweep — Spec-code consistency checker for rway
//!
//! Compares `docs/sway-spec.md` status markers against harness test results,
//! scans for TODO/FIXME comments, runs clippy, and detects unused public APIs.

use crate::{Harness, TestStatus};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ANSI colors
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecStatus {
    Implemented,
    Partial,
    Missing,
}

impl std::fmt::Display for SpecStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Implemented => write!(f, "IMPLEMENTED"),
            Self::Partial => write!(f, "PARTIAL"),
            Self::Missing => write!(f, "MISSING"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecEntry {
    pub line_number: usize,
    pub feature_name: String,
    pub normalized_key: String,
    pub status: SpecStatus,
}

#[derive(Debug)]
pub struct NeedsUpdateItem {
    pub spec_feature: String,
    pub spec_line: usize,
    pub old_status: SpecStatus,
    pub matching_tests: Vec<String>,
}

#[derive(Debug)]
pub struct DisagreesItem {
    pub spec_feature: String,
    pub spec_line: usize,
    pub not_impl_tests: Vec<String>,
}

#[derive(Debug)]
pub struct TodoEntry {
    pub file: String,
    pub line: usize,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Default)]
pub struct ClippySummary {
    pub by_crate: Vec<(String, usize)>,
    pub warning_types: Vec<String>,
    pub total: usize,
}

// ── Main entry ─────────────────────────────────────────────────────────

pub fn run_sweep(fix: bool) {
    let ws = workspace_root();
    let spec_path = ws.join("docs/sway-spec.md");

    println!("\n{BOLD}{CYAN}=== rway Entropy Sweep Report ==={RESET}\n");

    // 1. Parse spec
    let spec = parse_spec(&spec_path);
    println!("{DIM}  Parsed {} spec entries from sway-spec.md{RESET}", spec.len());

    // 2. Run harness and collect results
    let mut harness = Harness::new();
    harness.register_all();
    let report = harness.run();
    let all_details: Vec<_> = report
        .by_category
        .iter()
        .flat_map(|(_, cr)| cr.details.iter())
        .collect();
    println!(
        "{DIM}  Ran {} harness tests{RESET}\n",
        all_details.len()
    );

    // 3. Cross-reference
    let (consistent, needs_update, disagrees, unmatched) = cross_reference(&spec, &all_details);

    println!("{BOLD}Spec-Harness Consistency:{RESET}");
    println!("  {GREEN}✓{RESET} {consistent} items consistent");

    if needs_update.is_empty() {
        println!("  {GREEN}✓{RESET} 0 items need spec update");
    } else {
        println!(
            "  {YELLOW}⚠{RESET} {} items need spec update (MISSING/PARTIAL → IMPLEMENTED)",
            needs_update.len()
        );
        for item in &needs_update {
            let tests = item.matching_tests.join(", ");
            println!(
                "    - {name} [line {line}] ({old} → IMPLEMENTED, tests: {tests})",
                name = item.spec_feature,
                line = item.spec_line,
                old = item.old_status,
            );
        }
    }

    if disagrees.is_empty() {
        println!("  {GREEN}✓{RESET} 0 items spec claims implemented but harness disagrees");
    } else {
        println!(
            "  {RED}✗{RESET} {} items spec claims implemented but harness disagrees",
            disagrees.len()
        );
        for item in &disagrees {
            let tests = item.not_impl_tests.join(", ");
            println!(
                "    - {name} [line {line}] (tests: {tests})",
                name = item.spec_feature,
                line = item.spec_line,
            );
        }
    }

    if !unmatched.is_empty() {
        println!(
            "  {CYAN}i{RESET} {} passing tests with no matching spec entry",
            unmatched.len()
        );
        for name in &unmatched {
            println!("    - {name}");
        }
    }

    // 4. TODO/FIXME scan
    println!("\n{BOLD}TODO/FIXME Scan:{RESET}");
    let todos = scan_todos(&ws);
    let mut kind_counts: HashMap<&str, usize> = HashMap::new();
    for entry in &todos {
        *kind_counts.entry(entry.kind.as_str()).or_insert(0) += 1;
    }
    for kind in &["TODO", "FIXME", "HACK", "XXX"] {
        let count = kind_counts.get(kind).copied().unwrap_or(0);
        if count > 0 {
            println!("  Found {count} {kind} items in source code");
        }
    }
    if todos.is_empty() {
        println!("  {GREEN}✓ No TODO/FIXME/HACK/XXX found{RESET}");
    } else {
        for entry in &todos {
            println!(
                "    {DIM}{}:{}: [{}] {}{RESET}",
                entry.file, entry.line, entry.kind, entry.content
            );
        }
    }

    // 5. Clippy summary
    println!("\n{BOLD}Clippy Summary:{RESET}");
    let clippy = run_clippy(&ws);
    if clippy.total == 0 {
        println!("  {GREEN}✓ No clippy warnings{RESET}");
    } else {
        println!("  {YELLOW}{} total warning(s){RESET}", clippy.total);
        for (crate_name, count) in &clippy.by_crate {
            println!("    {crate_name}: {count} warning(s)");
        }
        if !clippy.warning_types.is_empty() {
            println!("  Warning types:");
            for wtype in &clippy.warning_types {
                println!("    - {wtype}");
            }
        }
    }

    // 6. Unused public APIs
    println!("\n{BOLD}Unused Public API (simplified):{RESET}");
    let unused = detect_unused_apis(&ws);
    if unused.is_empty() {
        println!("  {GREEN}✓ No obviously unused public APIs detected{RESET}");
    } else {
        println!(
            "  {YELLOW}{} potentially unused public item(s){RESET}",
            unused.len()
        );
        for (name, location) in &unused {
            println!("    - {name} ({location})");
        }
    }

    // 7. Auto-fix
    if fix {
        println!();
        if needs_update.is_empty() {
            println!("{GREEN}No spec updates needed.{RESET}");
        } else {
            println!("{BOLD}Applying spec fixes...{RESET}");
            let fixed = fix_spec(&spec_path, &needs_update);
            println!("{GREEN}✓ Updated {fixed} entries in sway-spec.md{RESET}");
        }
    }

    println!();
}

// ── Spec parsing ───────────────────────────────────────────────────────

fn parse_spec(path: &Path) -> Vec<SpecEntry> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not read spec file {}: {e}", path.display());
            return Vec::new();
        }
    };

    let mut entries = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;

        // Must be a table row with a status marker
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let status = if line.contains("[IMPLEMENTED]") {
            SpecStatus::Implemented
        } else if line.contains("[PARTIAL]") {
            SpecStatus::Partial
        } else if line.contains("[MISSING]") {
            SpecStatus::Missing
        } else {
            continue;
        };

        // Split by | and collect non-empty cells
        let cells: Vec<&str> = line
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if cells.len() < 2 {
            continue;
        }

        // Extract feature name from the best cell
        let feature_name = match extract_feature_name(&cells) {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };

        // Skip table header keywords
        if is_table_header(&feature_name) {
            continue;
        }

        let normalized_key = normalize_key(&feature_name);
        if normalized_key.is_empty() {
            continue;
        }

        entries.push(SpecEntry {
            line_number,
            feature_name,
            normalized_key,
            status,
        });
    }

    entries
}

/// Extract the feature/directive name from the table cells.
fn extract_feature_name(cells: &[&str]) -> Option<String> {
    // Determine which cell holds the directive name.
    // If the first cell is purely numeric / hex (type codes), use the second cell.
    let first_raw = cells[0].replace('`', "");
    let first_trimmed = first_raw.trim();
    let use_second = first_trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || c == 'x' || ('a'..='f').contains(&c));

    let name_cell = if use_second && cells.len() > 1 {
        cells[1]
    } else {
        cells[0]
    };

    // Extract text from backtick pairs and any trailing qualifier
    let parts: Vec<&str> = name_cell.split('`').collect();
    if parts.len() >= 3 {
        let bt_text = parts[1].trim();
        if !bt_text.is_empty() && !bt_text.starts_with('[') {
            // Grab qualifier text after closing backtick
            let after = if parts.len() > 2 { parts[2].trim() } else { "" };
            let qualifier: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphabetic() || c.is_whitespace() || *c == '_')
                .collect();
            let qualifier = qualifier.trim();
            return if qualifier.is_empty() {
                Some(bt_text.to_string())
            } else {
                Some(format!("{bt_text} {qualifier}"))
            };
        }
    }

    // No useful backtick content. If cell is CJK-heavy, try next cell.
    let has_ascii = first_trimmed.chars().any(|c| c.is_ascii_alphabetic());
    if !has_ascii && cells.len() > 1 {
        // Try second cell for backtick text
        let parts2: Vec<&str> = cells[1].split('`').collect();
        if parts2.len() >= 3 {
            let bt_text = parts2[1].trim();
            if !bt_text.is_empty() && !bt_text.starts_with('[') {
                // Take the first word/phrase from the backtick content
                let first_word: String = bt_text
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ' ')
                    .collect();
                let first_word = first_word.trim();
                if !first_word.is_empty() {
                    return Some(first_word.to_string());
                }
            }
        }
    }

    // Fallback: use the raw first cell text
    if has_ascii {
        Some(first_trimmed.to_string())
    } else {
        None
    }
}

fn is_table_header(name: &str) -> bool {
    let lower = name.to_lowercase();
    let headers = [
        "指令", "命令", "类型码", "条件属性", "标志", "事件码", "修饰键",
        "名称", "行为", "类型", "---", "描述",
    ];
    headers.iter().any(|h| lower.contains(h)) || lower.contains("---")
}

fn normalize_key(feature_name: &str) -> String {
    feature_name
        .to_lowercase()
        .replace([' ', '-', '/'], "_")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

// ── Cross-reference ────────────────────────────────────────────────────

fn cross_reference(
    spec: &[SpecEntry],
    tests: &[&crate::TestDetail],
) -> (usize, Vec<NeedsUpdateItem>, Vec<DisagreesItem>, Vec<String>) {
    let mut consistent = 0usize;
    let mut needs_update = Vec::new();
    let mut disagrees = Vec::new();
    let mut matched_tests: HashSet<String> = HashSet::new();

    for entry in spec {
        let matching: Vec<&&crate::TestDetail> = tests
            .iter()
            .filter(|t| test_matches_spec(t, &entry.normalized_key))
            .collect();

        if matching.is_empty() {
            // No matching tests — cannot determine consistency, count as consistent
            consistent += 1;
            continue;
        }

        for t in &matching {
            matched_tests.insert(t.name.clone());
        }

        let any_pass = matching.iter().any(|t| t.status.is_pass());
        let all_pass = matching
            .iter()
            .all(|t| t.status.is_pass() || matches!(t.status, TestStatus::Skip(_)));
        let all_not_impl = matching
            .iter()
            .all(|t| matches!(t.status, TestStatus::NotImplemented));

        match entry.status {
            SpecStatus::Implemented => {
                if all_not_impl {
                    disagrees.push(DisagreesItem {
                        spec_feature: entry.feature_name.clone(),
                        spec_line: entry.line_number,
                        not_impl_tests: matching.iter().map(|t| t.name.clone()).collect(),
                    });
                } else {
                    consistent += 1;
                }
            }
            SpecStatus::Missing => {
                if any_pass {
                    needs_update.push(NeedsUpdateItem {
                        spec_feature: entry.feature_name.clone(),
                        spec_line: entry.line_number,
                        old_status: SpecStatus::Missing,
                        matching_tests: matching
                            .iter()
                            .filter(|t| t.status.is_pass())
                            .map(|t| t.name.clone())
                            .collect(),
                    });
                } else {
                    consistent += 1;
                }
            }
            SpecStatus::Partial => {
                if all_pass {
                    needs_update.push(NeedsUpdateItem {
                        spec_feature: entry.feature_name.clone(),
                        spec_line: entry.line_number,
                        old_status: SpecStatus::Partial,
                        matching_tests: matching.iter().map(|t| t.name.clone()).collect(),
                    });
                } else {
                    consistent += 1;
                }
            }
        }
    }

    // Passing tests with no matching spec entry
    let unmatched: Vec<String> = tests
        .iter()
        .filter(|t| t.status.is_pass() && !matched_tests.contains(&t.name))
        .map(|t| t.name.clone())
        .collect();

    (consistent, needs_update, disagrees, unmatched)
}

fn test_matches_spec(test: &crate::TestDetail, spec_key: &str) -> bool {
    let test_name = test.name.to_lowercase();
    let feature = test
        .sway_feature
        .to_lowercase()
        .replace([' ', '-'], "_");

    // Word-boundary containment
    if contains_word(&test_name, spec_key) || contains_word(&feature, spec_key) {
        return true;
    }

    // Multi-word key: check if all significant words appear
    let words: Vec<&str> = spec_key.split('_').filter(|w| w.len() >= 3).collect();
    if words.len() > 1 && words.iter().all(|w| contains_word(&test_name, w)) {
        return true;
    }

    false
}

/// Check if `needle` appears in `haystack` at a word boundary (delimited by `_` or string edges).
fn contains_word(haystack: &str, needle: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0 || haystack.as_bytes()[abs_pos - 1] == b'_';
        let end_pos = abs_pos + needle.len();
        let after_ok =
            end_pos >= haystack.len() || haystack.as_bytes()[end_pos] == b'_';
        if before_ok && after_ok {
            return true;
        }
        start = abs_pos + 1;
    }
    false
}

// ── TODO/FIXME scan ────────────────────────────────────────────────────

fn scan_todos(ws: &Path) -> Vec<TodoEntry> {
    let output = Command::new("grep")
        .args([
            "-rn",
            "--include=*.rs",
            "--exclude-dir=target",
            "--exclude=sweep.rs",
            "-E",
            r"\b(TODO|FIXME|HACK|XXX)\b",
        ])
        .arg(".")
        .current_dir(ws)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("  Warning: grep failed: {e}");
            return Vec::new();
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        if line.contains("/target/") {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            continue;
        }
        let file = parts[0].strip_prefix("./").unwrap_or(parts[0]);
        let line_num = parts[1].parse::<usize>().unwrap_or(0);
        let content = parts[2].trim();

        let kind = if content.contains("FIXME") {
            "FIXME"
        } else if content.contains("HACK") {
            "HACK"
        } else if content.contains("XXX") {
            "XXX"
        } else {
            "TODO"
        };

        entries.push(TodoEntry {
            file: file.to_string(),
            line: line_num,
            kind: kind.to_string(),
            content: content.to_string(),
        });
    }

    entries
}

// ── Clippy ─────────────────────────────────────────────────────────────

fn run_clippy(ws: &Path) -> ClippySummary {
    let output = Command::new("cargo")
        .args(["clippy", "--workspace"])
        .current_dir(ws)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("  Warning: cargo clippy failed to run: {e}");
            return ClippySummary::default();
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut summary = ClippySummary::default();
    let mut crate_map: HashMap<String, usize> = HashMap::new();
    let mut type_set: HashSet<String> = HashSet::new();

    for line in stderr.lines() {
        // Summary line: "warning: `<crate>` (lib) generated N warning(s)"
        if line.starts_with("warning: `") && line.contains("generated") {
            if let Some((name, count)) = parse_clippy_summary_line(line) {
                *crate_map.entry(name).or_insert(0) += count;
                summary.total += count;
            }
            continue;
        }

        // Warning type: "= note: `#[warn(<type>)]`"
        if line.contains("#[warn(") {
            if let Some(wtype) = extract_warn_type(line) {
                type_set.insert(wtype);
            }
        }
    }

    summary.by_crate = {
        let mut v: Vec<_> = crate_map.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    };
    summary.warning_types = {
        let mut v: Vec<_> = type_set.into_iter().collect();
        v.sort();
        v
    };

    summary
}

fn parse_clippy_summary_line(line: &str) -> Option<(String, usize)> {
    // "warning: `rway-tiling` (lib) generated 1 warning"
    let rest = line.strip_prefix("warning: `")?;
    let end = rest.find('`')?;
    let crate_name = rest[..end].to_string();

    let gen_pos = line.find("generated ")?;
    let num_part = &line[gen_pos + "generated ".len()..];
    let num_str: String = num_part.chars().take_while(|c| c.is_ascii_digit()).collect();
    let count = num_str.parse::<usize>().ok()?;

    Some((crate_name, count))
}

fn extract_warn_type(line: &str) -> Option<String> {
    let start = line.find("#[warn(")?;
    let rest = &line[start + 7..];
    let end = rest.find(")]")?;
    Some(rest[..end].to_string())
}

// ── Unused public API ──────────────────────────────────────────────────

fn detect_unused_apis(ws: &Path) -> Vec<(String, String)> {
    let lib_crates = ["rway-tiling", "rway-config", "rway-ipc"];
    let consumer_dirs: Vec<PathBuf> = vec![ws.join("rway/src"), ws.join("rway-harness/src")];

    let mut unused = Vec::new();

    for lib_crate in &lib_crates {
        let src_dir = ws.join(lib_crate).join("src");
        if !src_dir.exists() {
            continue;
        }

        let pub_items = find_pub_items(&src_dir);

        for (name, file_loc) in &pub_items {
            // Skip very common / generic names that produce false positives
            if name.len() < 4 || matches!(name.as_str(), "new" | "default" | "from" | "into") {
                continue;
            }

            let mut referenced = false;
            for dir in &consumer_dirs {
                if dir.exists() && grep_quiet(dir, name) {
                    referenced = true;
                    break;
                }
            }
            // Also check within other lib crates
            if !referenced {
                for other_crate in &lib_crates {
                    if *other_crate == *lib_crate {
                        continue;
                    }
                    let other_src = ws.join(other_crate).join("src");
                    if other_src.exists() && grep_quiet(&other_src, name) {
                        referenced = true;
                        break;
                    }
                }
            }

            if !referenced {
                unused.push((name.clone(), format!("{lib_crate}/{file_loc}")));
            }
        }
    }

    unused
}

fn find_pub_items(dir: &Path) -> Vec<(String, String)> {
    let output = Command::new("grep")
        .args([
            "-rn",
            "--include=*.rs",
            "-E",
            r"^\s*pub (fn|struct|enum|trait|type) ",
        ])
        .arg(".")
        .current_dir(dir)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() < 3 {
            continue;
        }
        let file = parts[0].strip_prefix("./").unwrap_or(parts[0]);
        let content = parts[2].trim();

        if let Some(name) = extract_pub_name(content) {
            items.push((name, file.to_string()));
        }
    }

    items
}

fn extract_pub_name(line: &str) -> Option<String> {
    for kw in &["pub fn ", "pub struct ", "pub enum ", "pub trait ", "pub type "] {
        if let Some(pos) = line.find(kw) {
            let rest = &line[pos + kw.len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn grep_quiet(dir: &Path, pattern: &str) -> bool {
    let output = Command::new("grep")
        .args(["-rql", "--include=*.rs", pattern, "."])
        .current_dir(dir)
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

// ── Fix spec ───────────────────────────────────────────────────────────

fn fix_spec(path: &Path, updates: &[NeedsUpdateItem]) -> usize {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading spec for fix: {e}");
            return 0;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let mut fixed = 0;

    for update in updates {
        let idx = update.spec_line.wrapping_sub(1);
        if idx >= new_lines.len() {
            continue;
        }
        let line = &new_lines[idx];
        let old_marker = match update.old_status {
            SpecStatus::Missing => "`[MISSING]`",
            SpecStatus::Partial => "`[PARTIAL]`",
            SpecStatus::Implemented => continue,
        };
        if line.contains(old_marker) {
            new_lines[idx] = line.replace(old_marker, "`[IMPLEMENTED]`");
            fixed += 1;
            println!(
                "  {GREEN}✓{RESET} Line {}: {} ({} → IMPLEMENTED)",
                update.spec_line, update.spec_feature, update.old_status
            );
        }
    }

    if fixed > 0 {
        let mut new_content = new_lines.join("\n");
        if content.ends_with('\n') {
            new_content.push('\n');
        }
        if let Err(e) = fs::write(path, new_content) {
            eprintln!("Error writing spec: {e}");
            return 0;
        }
    }

    fixed
}

// ── Utilities ──────────────────────────────────────────────────────────

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // rway-harness/ → workspace root
    path
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_key_basic() {
        assert_eq!(normalize_key("set"), "set");
        assert_eq!(normalize_key("floating_modifier"), "floating_modifier");
        assert_eq!(normalize_key("focus direction"), "focus_direction");
        assert_eq!(normalize_key("move to workspace"), "move_to_workspace");
        assert_eq!(normalize_key("GET_WORKSPACES"), "get_workspaces");
    }

    #[test]
    fn contains_word_matches() {
        assert!(contains_word("parse_set_variable", "set"));
        assert!(contains_word("parse_floating_modifier", "floating_modifier"));
        assert!(contains_word("set", "set"));
        assert!(!contains_word("preset_value", "set")); // not at word boundary
    }

    #[test]
    fn contains_word_boundary() {
        assert!(contains_word("parse_exec_always", "exec"));
        assert!(contains_word("parse_exec_always", "exec_always"));
        assert!(contains_word("exec", "exec"));
        assert!(!contains_word("execute", "exec"));
    }

    #[test]
    fn extract_feature_from_backticks() {
        let cells = vec!["`set`", "`set $<name> <value>`", "-", "`[IMPLEMENTED]`", "P0"];
        assert_eq!(extract_feature_name(&cells), Some("set".to_string()));
    }

    #[test]
    fn extract_feature_with_qualifier() {
        let cells = vec!["`focus` direction", "`focus up\\|right`", "`[IMPLEMENTED]`", "P0"];
        assert_eq!(
            extract_feature_name(&cells),
            Some("focus direction".to_string())
        );
    }

    #[test]
    fn extract_feature_numeric_first_cell() {
        let cells = vec!["1", "`GET_WORKSPACES`", "`[IMPLEMENTED]`", "P0"];
        assert_eq!(
            extract_feature_name(&cells),
            Some("GET_WORKSPACES".to_string())
        );
    }

    #[test]
    fn parse_spec_does_not_panic() {
        let ws = workspace_root();
        let spec_path = ws.join("docs/sway-spec.md");
        if spec_path.exists() {
            let entries = parse_spec(&spec_path);
            assert!(!entries.is_empty());
        }
    }

    #[test]
    fn spec_status_display() {
        assert_eq!(format!("{}", SpecStatus::Implemented), "IMPLEMENTED");
        assert_eq!(format!("{}", SpecStatus::Partial), "PARTIAL");
        assert_eq!(format!("{}", SpecStatus::Missing), "MISSING");
    }

    #[test]
    fn fix_spec_does_not_panic_on_empty_updates() {
        let ws = workspace_root();
        let spec_path = ws.join("docs/sway-spec.md");
        let empty: Vec<NeedsUpdateItem> = Vec::new();
        let fixed = fix_spec(&spec_path, &empty);
        assert_eq!(fixed, 0);
    }
}
