//! 终端彩色报告输出

use crate::{CompatReport, TestStatus};

// ANSI 颜色码
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// 将报告输出到 stdout
pub fn print_report(report: &CompatReport) {
    let mut output = String::new();
    format_report(report, &mut output);
    print!("{output}");
}

/// 将报告格式化到字符串
pub fn format_report(report: &CompatReport, out: &mut String) {
    // 标题
    out.push_str(&format!(
        "\n{BOLD}{CYAN}\
         +==================================================+\n\
         |         rway Sway Compatibility Report            |\n\
         +==================================================+{RESET}\n\n"
    ));

    // 分类表头
    out.push_str(&format!(
        "{BOLD}  {:<14} {:>6} {:>6} {:>6} {:>6} {:>7}{RESET}\n",
        "Category", "Pass", "Fail", "Skip", "N/A", "Rate"
    ));
    out.push_str(&format!(
        "  {}\n",
        "-".repeat(50)
    ));

    // 各分类行
    for (category, cat_report) in &report.by_category {
        let rate = cat_report.pass_rate();
        let rate_color = if rate >= 80.0 {
            GREEN
        } else if rate >= 50.0 {
            YELLOW
        } else {
            RED
        };

        out.push_str(&format!(
            "  {:<14} {GREEN}{:>6}{RESET} {RED}{:>6}{RESET} {YELLOW}{:>6}{RESET} {DIM}{:>6}{RESET} {rate_color}{:>6.0}%{RESET}\n",
            format!("{category}"),
            cat_report.passed,
            cat_report.failed,
            cat_report.skipped,
            cat_report.not_implemented,
            rate,
        ));
    }

    // 总计行
    let total_rate = report.pass_rate();
    let total_color = if total_rate >= 80.0 {
        GREEN
    } else if total_rate >= 50.0 {
        YELLOW
    } else {
        RED
    };

    out.push_str(&format!(
        "  {}\n",
        "-".repeat(50)
    ));
    out.push_str(&format!(
        "  {BOLD}{:<14}{RESET} {GREEN}{:>6}{RESET} {RED}{:>6}{RESET} {YELLOW}{:>6}{RESET} {DIM}{:>6}{RESET} {total_color}{BOLD}{:>6.0}%{RESET}\n",
        "TOTAL",
        report.passed,
        report.failed,
        report.skipped,
        report.not_implemented,
        total_rate,
    ));

    out.push('\n');

    // 失败详情
    let failures: Vec<_> = report
        .by_category
        .iter()
        .flat_map(|(cat, r)| {
            r.details.iter().filter_map(move |d| {
                if let TestStatus::Fail(reason) = &d.status {
                    Some((cat, d, reason.clone()))
                } else {
                    None
                }
            })
        })
        .collect();

    if !failures.is_empty() {
        out.push_str(&format!(
            "{BOLD}{RED}  Failed Tests:{RESET}\n"
        ));
        for (cat, detail, reason) in &failures {
            out.push_str(&format!(
                "    {RED}FAIL{RESET} [{cat}] {name} ({feature}): {reason}\n",
                name = detail.name,
                feature = detail.sway_feature,
            ));
        }
        out.push('\n');
    }

    // 汇总
    out.push_str(&format!(
        "  {BOLD}Summary:{RESET} {total} tests | \
         {GREEN}{pass} passed{RESET} | \
         {RED}{fail} failed{RESET} | \
         {YELLOW}{skip} skipped{RESET} | \
         {DIM}{ni} not implemented{RESET}\n\n",
        total = report.total,
        pass = report.passed,
        fail = report.failed,
        skip = report.skipped,
        ni = report.not_implemented,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, CategoryReport, TestDetail, Priority};

    #[test]
    fn format_report_does_not_panic_on_empty() {
        let report = CompatReport {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            not_implemented: 0,
            by_category: vec![],
        };
        let mut out = String::new();
        format_report(&report, &mut out);
        assert!(out.contains("rway Sway Compatibility Report"));
        assert!(out.contains("TOTAL"));
    }

    #[test]
    fn format_report_shows_categories() {
        let report = CompatReport {
            total: 3,
            passed: 2,
            failed: 1,
            skipped: 0,
            not_implemented: 0,
            by_category: vec![(
                Category::Config,
                CategoryReport {
                    passed: 2,
                    failed: 1,
                    skipped: 0,
                    not_implemented: 0,
                    details: vec![
                        TestDetail {
                            name: "parse_set".into(),
                            sway_feature: "set variable".into(),
                            priority: Priority::P0,
                            status: TestStatus::Pass,
                        },
                        TestDetail {
                            name: "parse_bindsym".into(),
                            sway_feature: "bindsym".into(),
                            priority: Priority::P0,
                            status: TestStatus::Pass,
                        },
                        TestDetail {
                            name: "parse_mode".into(),
                            sway_feature: "mode block".into(),
                            priority: Priority::P1,
                            status: TestStatus::Fail("mode blocks not supported".into()),
                        },
                    ],
                },
            )],
        };
        let mut out = String::new();
        format_report(&report, &mut out);
        assert!(out.contains("Config"));
        assert!(out.contains("Failed Tests"));
        assert!(out.contains("parse_mode"));
    }

    #[test]
    fn format_report_no_failures_section_when_all_pass() {
        let report = CompatReport {
            total: 1,
            passed: 1,
            failed: 0,
            skipped: 0,
            not_implemented: 0,
            by_category: vec![(
                Category::Tiling,
                CategoryReport {
                    passed: 1,
                    failed: 0,
                    skipped: 0,
                    not_implemented: 0,
                    details: vec![TestDetail {
                        name: "split_h".into(),
                        sway_feature: "splith layout".into(),
                        priority: Priority::P0,
                        status: TestStatus::Pass,
                    }],
                },
            )],
        };
        let mut out = String::new();
        format_report(&report, &mut out);
        assert!(!out.contains("Failed Tests"));
    }
}
