//! Regression baseline 读写与对比
//!
//! 用于检测兼容性测试是否发生回归（pass count 下降）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::CompatReport;

/// 基线快照
#[derive(Debug, Serialize, Deserialize)]
pub struct Baseline {
    pub timestamp: String,
    pub categories: HashMap<String, CategoryBaseline>,
}

/// 单个分类的基线数据
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryBaseline {
    pub passed: usize,
    pub total: usize,
}

/// 回归条目
#[derive(Debug)]
pub struct RegressionItem {
    pub category: String,
    pub old_passed: usize,
    pub new_passed: usize,
}

/// 从文件加载基线，不存在时返回 None
pub fn load_baseline(path: &Path) -> Option<Baseline> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 保存基线到文件
pub fn save_baseline(path: &Path, baseline: &Baseline) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(baseline).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// 从报告生成基线
pub fn baseline_from_report(report: &CompatReport) -> Baseline {
    let mut categories = HashMap::new();
    for (cat, cat_report) in &report.by_category {
        categories.insert(
            cat.to_string(),
            CategoryBaseline {
                passed: cat_report.passed,
                total: cat_report.total(),
            },
        );
    }
    Baseline {
        timestamp: chrono_now(),
        categories,
    }
}

/// 对比基线与当前报告，返回回归列表
pub fn check_regression(old: &Baseline, report: &CompatReport) -> Vec<RegressionItem> {
    let mut regressions = Vec::new();
    for (cat, cat_report) in &report.by_category {
        let cat_name = cat.to_string();
        if let Some(old_cat) = old.categories.get(&cat_name) {
            if cat_report.passed < old_cat.passed {
                regressions.push(RegressionItem {
                    category: cat_name,
                    old_passed: old_cat.passed,
                    new_passed: cat_report.passed,
                });
            }
        }
    }
    regressions
}

/// 简易时间戳（避免引入 chrono 依赖）
fn chrono_now() -> String {
    // 使用 UNIX timestamp 格式
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("unix:{}", d.as_secs()),
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_roundtrip() {
        let mut categories = HashMap::new();
        categories.insert(
            "Config".to_string(),
            CategoryBaseline {
                passed: 10,
                total: 20,
            },
        );
        let baseline = Baseline {
            timestamp: "test".to_string(),
            categories,
        };

        let json = serde_json::to_string(&baseline).unwrap();
        let restored: Baseline = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.categories["Config"].passed, 10);
        assert_eq!(restored.categories["Config"].total, 20);
    }

    #[test]
    fn check_regression_detects_drop() {
        let mut old_cats = HashMap::new();
        old_cats.insert(
            "Config".to_string(),
            CategoryBaseline {
                passed: 15,
                total: 20,
            },
        );
        let old = Baseline {
            timestamp: "old".to_string(),
            categories: old_cats,
        };

        let report = CompatReport {
            total: 20,
            passed: 12,
            failed: 8,
            skipped: 0,
            not_implemented: 0,
            by_category: vec![(
                crate::Category::Config,
                crate::CategoryReport {
                    passed: 12,
                    failed: 8,
                    skipped: 0,
                    not_implemented: 0,
                    details: vec![],
                },
            )],
        };

        let regressions = check_regression(&old, &report);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].old_passed, 15);
        assert_eq!(regressions[0].new_passed, 12);
    }

    #[test]
    fn check_regression_passes_when_improved() {
        let mut old_cats = HashMap::new();
        old_cats.insert(
            "Config".to_string(),
            CategoryBaseline {
                passed: 10,
                total: 20,
            },
        );
        let old = Baseline {
            timestamp: "old".to_string(),
            categories: old_cats,
        };

        let report = CompatReport {
            total: 20,
            passed: 15,
            failed: 5,
            skipped: 0,
            not_implemented: 0,
            by_category: vec![(
                crate::Category::Config,
                crate::CategoryReport {
                    passed: 15,
                    failed: 5,
                    skipped: 0,
                    not_implemented: 0,
                    details: vec![],
                },
            )],
        };

        let regressions = check_regression(&old, &report);
        assert!(regressions.is_empty());
    }
}
