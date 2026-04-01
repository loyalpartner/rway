//! rway Sway 兼容性测试框架
//!
//! 提供结构化的兼容性测试注册、运行和报告生成。

pub mod baseline;
pub mod categories;
pub mod report;
pub mod sweep;

use std::fmt;

/// 测试分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Config,
    Ipc,
    Tiling,
    Window,
    Workspace,
    Input,
    Output,
    Appearance,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Config => write!(f, "Config"),
            Category::Ipc => write!(f, "IPC"),
            Category::Tiling => write!(f, "Tiling"),
            Category::Window => write!(f, "Window"),
            Category::Workspace => write!(f, "Workspace"),
            Category::Input => write!(f, "Input"),
            Category::Output => write!(f, "Output"),
            Category::Appearance => write!(f, "Appearance"),
        }
    }
}

impl Category {
    pub fn all() -> &'static [Category] {
        &[
            Category::Config,
            Category::Ipc,
            Category::Tiling,
            Category::Window,
            Category::Workspace,
            Category::Input,
            Category::Output,
            Category::Appearance,
        ]
    }
}

/// 优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    P0,
    P1,
    P2,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
        }
    }
}

/// 测试结果状态
#[derive(Debug, Clone)]
pub enum TestStatus {
    Pass,
    Fail(String),
    Skip(String),
    NotImplemented,
}

impl TestStatus {
    pub fn is_pass(&self) -> bool {
        matches!(self, TestStatus::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, TestStatus::Fail(_))
    }
}

/// 单个兼容性测试
pub struct CompatTest {
    pub category: Category,
    pub name: String,
    pub description: String,
    pub sway_feature: String,
    pub priority: Priority,
    pub test_fn: Box<dyn Fn() -> TestStatus>,
}

/// 单个分类的报告
#[derive(Debug, Clone, Default)]
pub struct CategoryReport {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub not_implemented: usize,
    pub details: Vec<TestDetail>,
}

impl CategoryReport {
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped + self.not_implemented
    }

    pub fn pass_rate(&self) -> f64 {
        let testable = self.passed + self.failed;
        if testable == 0 {
            return 0.0;
        }
        self.passed as f64 / testable as f64 * 100.0
    }
}

/// 单个测试的详细结果
#[derive(Debug, Clone)]
pub struct TestDetail {
    pub name: String,
    pub sway_feature: String,
    pub priority: Priority,
    pub status: TestStatus,
}

/// 兼容性报告
#[derive(Debug, Clone)]
pub struct CompatReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub not_implemented: usize,
    pub by_category: Vec<(Category, CategoryReport)>,
}

impl CompatReport {
    pub fn pass_rate(&self) -> f64 {
        let testable = self.passed + self.failed;
        if testable == 0 {
            return 0.0;
        }
        self.passed as f64 / testable as f64 * 100.0
    }
}

/// 测试 Harness：注册并运行兼容性测试
pub struct Harness {
    tests: Vec<CompatTest>,
}

impl Harness {
    pub fn new() -> Self {
        Harness { tests: Vec::new() }
    }

    pub fn register(&mut self, test: CompatTest) {
        self.tests.push(test);
    }

    /// 运行所有已注册的测试
    pub fn run(&self) -> CompatReport {
        self.run_filtered(|_| true)
    }

    /// 仅运行指定分类的测试
    pub fn run_category(&self, cat: Category) -> CompatReport {
        self.run_filtered(|t| t.category == cat)
    }

    fn run_filtered(&self, predicate: impl Fn(&CompatTest) -> bool) -> CompatReport {
        let mut by_category: Vec<(Category, CategoryReport)> = Category::all()
            .iter()
            .map(|&c| (c, CategoryReport::default()))
            .collect();

        let mut total = 0usize;
        let mut passed = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;
        let mut not_implemented = 0usize;

        for test in &self.tests {
            if !predicate(test) {
                continue;
            }

            let status = (test.test_fn)();
            total += 1;

            match &status {
                TestStatus::Pass => passed += 1,
                TestStatus::Fail(_) => failed += 1,
                TestStatus::Skip(_) => skipped += 1,
                TestStatus::NotImplemented => not_implemented += 1,
            }

            let detail = TestDetail {
                name: test.name.clone(),
                sway_feature: test.sway_feature.clone(),
                priority: test.priority,
                status: status.clone(),
            };

            if let Some((_, cat_report)) = by_category.iter_mut().find(|(c, _)| *c == test.category) {
                match &detail.status {
                    TestStatus::Pass => cat_report.passed += 1,
                    TestStatus::Fail(_) => cat_report.failed += 1,
                    TestStatus::Skip(_) => cat_report.skipped += 1,
                    TestStatus::NotImplemented => cat_report.not_implemented += 1,
                }
                cat_report.details.push(detail);
            }
        }

        // 过滤掉没有测试的分类
        by_category.retain(|(_, r)| r.total() > 0);

        CompatReport {
            total,
            passed,
            failed,
            skipped,
            not_implemented,
            by_category,
        }
    }

    /// 注册所有内置测试
    pub fn register_all(&mut self) {
        categories::config::register(self);
        categories::ipc::register(self);
        categories::tiling::register(self);
        categories::window::register(self);
        categories::workspace::register(self);
        categories::input::register(self);
        categories::output::register(self);
        categories::appearance::register(self);
    }
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_new_has_no_tests() {
        let h = Harness::new();
        let report = h.run();
        assert_eq!(report.total, 0);
    }

    #[test]
    fn harness_register_and_run_pass() {
        let mut h = Harness::new();
        h.register(CompatTest {
            category: Category::Config,
            name: "test_pass".into(),
            description: "A passing test".into(),
            sway_feature: "set variable".into(),
            priority: Priority::P0,
            test_fn: Box::new(|| TestStatus::Pass),
        });
        let report = h.run();
        assert_eq!(report.total, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn harness_register_and_run_fail() {
        let mut h = Harness::new();
        h.register(CompatTest {
            category: Category::Ipc,
            name: "test_fail".into(),
            description: "A failing test".into(),
            sway_feature: "get_version".into(),
            priority: Priority::P0,
            test_fn: Box::new(|| TestStatus::Fail("not implemented".into())),
        });
        let report = h.run();
        assert_eq!(report.total, 1);
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn harness_run_category_filters() {
        let mut h = Harness::new();
        h.register(CompatTest {
            category: Category::Config,
            name: "config_test".into(),
            description: "".into(),
            sway_feature: "".into(),
            priority: Priority::P0,
            test_fn: Box::new(|| TestStatus::Pass),
        });
        h.register(CompatTest {
            category: Category::Ipc,
            name: "ipc_test".into(),
            description: "".into(),
            sway_feature: "".into(),
            priority: Priority::P1,
            test_fn: Box::new(|| TestStatus::Pass),
        });

        let config_report = h.run_category(Category::Config);
        assert_eq!(config_report.total, 1);
        assert_eq!(config_report.passed, 1);

        let ipc_report = h.run_category(Category::Ipc);
        assert_eq!(ipc_report.total, 1);
    }

    #[test]
    fn harness_report_pass_rate() {
        let mut h = Harness::new();
        h.register(CompatTest {
            category: Category::Config,
            name: "pass".into(),
            description: "".into(),
            sway_feature: "".into(),
            priority: Priority::P0,
            test_fn: Box::new(|| TestStatus::Pass),
        });
        h.register(CompatTest {
            category: Category::Config,
            name: "fail".into(),
            description: "".into(),
            sway_feature: "".into(),
            priority: Priority::P0,
            test_fn: Box::new(|| TestStatus::Fail("reason".into())),
        });
        let report = h.run();
        assert!((report.pass_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", Category::Config), "Config");
        assert_eq!(format!("{}", Category::Ipc), "IPC");
        assert_eq!(format!("{}", Category::Tiling), "Tiling");
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::P0 < Priority::P1);
        assert!(Priority::P1 < Priority::P2);
    }

    #[test]
    fn test_status_helpers() {
        assert!(TestStatus::Pass.is_pass());
        assert!(!TestStatus::Pass.is_fail());
        assert!(TestStatus::Fail("x".into()).is_fail());
        assert!(!TestStatus::NotImplemented.is_pass());
    }

    #[test]
    fn category_report_pass_rate_no_testable() {
        let r = CategoryReport {
            passed: 0,
            failed: 0,
            skipped: 5,
            not_implemented: 3,
            details: vec![],
        };
        assert!((r.pass_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn register_all_does_not_panic() {
        let mut h = Harness::new();
        h.register_all();
        let report = h.run();
        assert!(report.total > 0);
    }
}
