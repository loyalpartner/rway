//! rway Sway 兼容性测试 CLI
//!
//! 用法:
//!   cargo run -p rway-harness -- report              # 完整报告
//!   cargo run -p rway-harness -- report config        # 仅 Config 分类
//!   cargo run -p rway-harness -- report --check-regression  # 回归检测
//!   cargo run -p rway-harness -- sweep               # 一致性检查
//!   cargo run -p rway-harness -- sweep --fix          # 自动修复 spec 状态

use rway_harness::baseline;
use rway_harness::report::print_report;
use rway_harness::sweep;
use rway_harness::{Category, Harness};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut harness = Harness::new();
    harness.register_all();

    let command = args.get(1).map(|s| s.as_str()).unwrap_or("report");

    match command {
        "report" => {
            let mut category_filter: Option<&str> = None;
            let mut check_regression = false;

            for arg in args.iter().skip(2) {
                match arg.as_str() {
                    "--check-regression" => check_regression = true,
                    other => {
                        if category_filter.is_none() {
                            category_filter = Some(other);
                        }
                    }
                }
            }

            let report = match category_filter {
                Some(cat_str) => {
                    if let Some(cat) = parse_category(cat_str) {
                        harness.run_category(cat)
                    } else {
                        eprintln!("Unknown category: {cat_str}");
                        eprintln!(
                            "Available: config, ipc, tiling, window, workspace, input, output, appearance"
                        );
                        std::process::exit(1);
                    }
                }
                None => harness.run(),
            };

            print_report(&report);

            if check_regression {
                run_regression_check(&report);
            }
        }
        "sweep" => {
            let fix = args.iter().any(|a| a == "--fix");
            sweep::run_sweep(fix);
        }
        "help" | "--help" | "-h" => {
            println!("Usage: rway-harness <command> [options]");
            println!();
            println!("Commands:");
            println!("  report                        Run all compatibility tests");
            println!("  report <category>             Run tests for a specific category");
            println!("  report --check-regression     Run tests and check for regressions");
            println!("  sweep                         Spec-code consistency check");
            println!("  sweep --fix                   Auto-fix spec status markers");
            println!();
            println!(
                "Categories: config, ipc, tiling, window, workspace, input, output, appearance"
            );
        }
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Run with --help for usage");
            std::process::exit(1);
        }
    }
}

fn run_regression_check(report: &rway_harness::CompatReport) {
    let baseline_path = baseline_file_path();

    match baseline::load_baseline(&baseline_path) {
        Some(old_baseline) => {
            let regressions = baseline::check_regression(&old_baseline, report);
            if regressions.is_empty() {
                println!("\x1b[32m✓ No regressions detected.\x1b[0m");
                let new_baseline = baseline::baseline_from_report(report);
                if let Err(e) = baseline::save_baseline(&baseline_path, &new_baseline) {
                    eprintln!("Warning: failed to update baseline: {e}");
                } else {
                    println!("  Baseline updated: {}", baseline_path.display());
                }
            } else {
                eprintln!("\x1b[31m✗ Regressions detected!\x1b[0m");
                for r in &regressions {
                    eprintln!(
                        "  {}: passed {} → {} (was {})",
                        r.category, r.old_passed, r.new_passed, r.old_passed
                    );
                }
                std::process::exit(1);
            }
        }
        None => {
            println!("No baseline found. Creating initial baseline...");
            let new_baseline = baseline::baseline_from_report(report);
            if let Err(e) = baseline::save_baseline(&baseline_path, &new_baseline) {
                eprintln!("Error: failed to create baseline: {e}");
                std::process::exit(1);
            }
            println!(
                "\x1b[32m✓ Initial baseline created: {}\x1b[0m",
                baseline_path.display()
            );
        }
    }
}

fn baseline_file_path() -> PathBuf {
    // baseline.json 放在 rway-harness crate 目录下
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("baseline.json");
    path
}

fn parse_category(s: &str) -> Option<Category> {
    match s.to_lowercase().as_str() {
        "config" => Some(Category::Config),
        "ipc" => Some(Category::Ipc),
        "tiling" => Some(Category::Tiling),
        "window" => Some(Category::Window),
        "workspace" => Some(Category::Workspace),
        "input" => Some(Category::Input),
        "output" => Some(Category::Output),
        "appearance" => Some(Category::Appearance),
        _ => None,
    }
}
