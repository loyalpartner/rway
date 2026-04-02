// animation.rs — 窗口动画系统
// 在平铺布局变化时，对窗口位置和大小进行平滑插值过渡

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 动画配置
pub struct AnimationConfig {
    /// 动画持续时间（默认 200ms）
    pub duration: Duration,
    /// 是否启用动画
    pub enabled: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
            enabled: true,
        }
    }
}

/// 单个窗口的动画状态
struct WindowAnimation {
    start_x: f64,
    start_y: f64,
    start_w: f64,
    start_h: f64,
    target_x: f64,
    target_y: f64,
    target_w: f64,
    target_h: f64,
    start_time: Instant,
    duration: Duration,
}

impl WindowAnimation {
    /// 计算当前帧的插值位置
    ///
    /// 使用 ease-out cubic 缓动：t' = 1 - (1-t)^3
    /// 动画开始快、结尾慢，手感自然
    fn interpolate(&self, now: Instant) -> (i32, i32, i32, i32) {
        let elapsed = now.duration_since(self.start_time);
        let t = (elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0);
        // ease-out cubic: 快速到达目标附近，最后减速停下
        let t = 1.0 - (1.0 - t).powi(3);

        let x = self.start_x + (self.target_x - self.start_x) * t;
        let y = self.start_y + (self.target_y - self.start_y) * t;
        let w = self.start_w + (self.target_w - self.start_w) * t;
        let h = self.start_h + (self.target_h - self.start_h) * t;
        (
            x.round() as i32,
            y.round() as i32,
            w.round() as i32,
            h.round() as i32,
        )
    }

    /// 动画是否已完成
    fn is_finished(&self, now: Instant) -> bool {
        now.duration_since(self.start_time) >= self.duration
    }
}

/// 动画管理器：跟踪所有窗口的动画状态
pub struct AnimationManager {
    config: AnimationConfig,
    /// 活跃的动画（window_id -> 动画状态）
    animations: HashMap<u64, WindowAnimation>,
    /// 每个窗口的当前渲染位置（动画中间帧时与目标不同）
    current_positions: HashMap<u64, (i32, i32, i32, i32)>,
}

impl AnimationManager {
    pub fn new(config: AnimationConfig) -> Self {
        Self {
            config,
            animations: HashMap::new(),
            current_positions: HashMap::new(),
        }
    }

    /// 设置窗口的目标几何位置
    ///
    /// 如果启用动画且窗口位置/大小发生变化，启动新的插值动画。
    /// 新窗口（第一次出现）不做动画，直接到达目标位置。
    pub fn set_target(
        &mut self,
        window_id: u64,
        target_x: i32,
        target_y: i32,
        target_w: i32,
        target_h: i32,
    ) {
        if !self.config.enabled {
            // 动画禁用时直接跳到目标位置
            self.current_positions
                .insert(window_id, (target_x, target_y, target_w, target_h));
            return;
        }

        // 新窗口（第一次出现）不做位置动画，直接到位
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.current_positions.entry(window_id)
        {
            e.insert((target_x, target_y, target_w, target_h));
            return;
        }

        // 获取当前位置（如果有动画在进行则用当前插值位置）
        let (cx, cy, cw, ch) = self
            .current_positions
            .get(&window_id)
            .copied()
            .unwrap_or((target_x, target_y, target_w, target_h));

        // 如果位置没变，不启动动画
        if cx == target_x && cy == target_y && cw == target_w && ch == target_h {
            return;
        }

        self.animations.insert(
            window_id,
            WindowAnimation {
                start_x: cx as f64,
                start_y: cy as f64,
                start_w: cw as f64,
                start_h: ch as f64,
                target_x: target_x as f64,
                target_y: target_y as f64,
                target_w: target_w as f64,
                target_h: target_h as f64,
                start_time: Instant::now(),
                duration: self.config.duration,
            },
        );
    }

    /// 每帧调用：更新所有活跃动画的插值位置
    ///
    /// 返回是否还有活跃的动画（用于判断是否需要继续请求重绘）
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        let mut any_active = false;

        // 更新所有活跃动画的当前位置
        let mut finished = Vec::new();
        for (&window_id, anim) in &self.animations {
            let pos = anim.interpolate(now);
            self.current_positions.insert(window_id, pos);
            if anim.is_finished(now) {
                finished.push(window_id);
            } else {
                any_active = true;
            }
        }

        // 移除已完成的动画
        for id in finished {
            self.animations.remove(&id);
        }

        any_active
    }

    /// 获取窗口当前应渲染的位置 (x, y, w, h)
    pub fn get_position(&self, window_id: u64) -> Option<(i32, i32, i32, i32)> {
        self.current_positions.get(&window_id).copied()
    }

    /// 窗口被移除时清理其动画状态
    pub fn remove(&mut self, window_id: u64) {
        self.animations.remove(&window_id);
        self.current_positions.remove(&window_id);
    }

    /// 是否有活跃的动画正在播放
    pub fn has_active_animations(&self) -> bool {
        !self.animations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// 禁用动画时，set_target 应直接跳到目标位置
    #[test]
    fn test_animation_disabled_jumps_immediately() {
        let config = AnimationConfig {
            enabled: false,
            duration: Duration::from_millis(200),
        };
        let mut mgr = AnimationManager::new(config);

        mgr.set_target(1, 100, 200, 800, 600);
        assert_eq!(mgr.get_position(1), Some((100, 200, 800, 600)));

        // 即使改变目标位置也应立即跳转
        mgr.set_target(1, 300, 400, 1024, 768);
        assert_eq!(mgr.get_position(1), Some((300, 400, 1024, 768)));

        // 不应有活跃动画
        assert!(!mgr.has_active_animations());
    }

    /// 新窗口（第一次出现）不应启动动画，直接到达目标位置
    #[test]
    fn test_animation_new_window_no_animation() {
        let config = AnimationConfig {
            enabled: true,
            duration: Duration::from_millis(200),
        };
        let mut mgr = AnimationManager::new(config);

        mgr.set_target(1, 100, 200, 800, 600);

        // 新窗口应直接到位，无动画
        assert_eq!(mgr.get_position(1), Some((100, 200, 800, 600)));
        assert!(!mgr.has_active_animations());
    }

    /// 动画开始时（t ≈ 0），位置应接近起始位置
    #[test]
    fn test_animation_interpolation_at_start() {
        let config = AnimationConfig {
            enabled: true,
            duration: Duration::from_millis(500),
        };
        let mut mgr = AnimationManager::new(config);

        // 先设置初始位置（第一次出现，不触发动画）
        mgr.set_target(1, 0, 0, 100, 100);

        // 改变位置，触发动画
        mgr.set_target(1, 1000, 1000, 500, 500);

        // 立即 tick（t ≈ 0），应接近起始位置
        mgr.tick();
        let (x, y, w, h) = mgr.get_position(1).unwrap();

        // ease-out cubic 在 t=0 时为 0，所以应该还在起始位置附近
        assert!(x < 100, "x={} 应接近起始位置 0", x);
        assert!(y < 100, "y={} 应接近起始位置 0", y);
        assert!(w < 200, "w={} 应接近起始大小 100", w);
        assert!(h < 200, "h={} 应接近起始大小 100", h);
    }

    /// 动画结束后（t >= 1.0），位置应精确等于目标位置
    #[test]
    fn test_animation_interpolation_at_end() {
        let config = AnimationConfig {
            enabled: true,
            duration: Duration::from_millis(10), // 极短动画
        };
        let mut mgr = AnimationManager::new(config);

        mgr.set_target(1, 0, 0, 100, 100);
        mgr.set_target(1, 500, 500, 300, 300);

        // 等待动画完成
        thread::sleep(Duration::from_millis(20));
        mgr.tick();

        assert_eq!(mgr.get_position(1), Some((500, 500, 300, 300)));
    }

    /// 验证 ease-out cubic 缓动函数的特性：
    /// 中间时刻 t=0.5 时 t' = 1-(1-0.5)^3 = 0.875，已走过 87.5% 的距离
    #[test]
    fn test_animation_ease_out_cubic() {
        // 直接测试 WindowAnimation 的插值数学
        let now = Instant::now();
        let anim = WindowAnimation {
            start_x: 0.0,
            start_y: 0.0,
            start_w: 100.0,
            start_h: 100.0,
            target_x: 1000.0,
            target_y: 1000.0,
            target_w: 500.0,
            target_h: 500.0,
            start_time: now,
            duration: Duration::from_millis(1000),
        };

        // 在 t=0.5（500ms）时，ease-out cubic 应已走过约 87.5%
        let halfway = now + Duration::from_millis(500);
        let (x, y, _w, _h) = anim.interpolate(halfway);

        // 87.5% of 1000 = 875
        assert!(
            (x - 875).abs() <= 1,
            "ease-out cubic 在 t=0.5 时 x={} 应约为 875",
            x
        );
        assert!(
            (y - 875).abs() <= 1,
            "ease-out cubic 在 t=0.5 时 y={} 应约为 875",
            y
        );

        // t=1.0 时应精确到达目标
        let end = now + Duration::from_millis(1000);
        let (x, y, w, h) = anim.interpolate(end);
        assert_eq!((x, y, w, h), (1000, 1000, 500, 500));

        // t=0.0 时应在起始位置
        let (x, y, w, h) = anim.interpolate(now);
        assert_eq!((x, y, w, h), (0, 0, 100, 100));
    }

    /// tick 应自动移除已完成的动画
    #[test]
    fn test_animation_tick_removes_finished() {
        let config = AnimationConfig {
            enabled: true,
            duration: Duration::from_millis(5),
        };
        let mut mgr = AnimationManager::new(config);

        mgr.set_target(1, 0, 0, 100, 100);
        mgr.set_target(1, 500, 500, 300, 300);
        assert!(mgr.has_active_animations());

        // 等待动画完成
        thread::sleep(Duration::from_millis(10));
        let still_active = mgr.tick();

        assert!(!still_active, "tick 返回 false 表示无活跃动画");
        assert!(!mgr.has_active_animations());

        // 位置应精确等于目标
        assert_eq!(mgr.get_position(1), Some((500, 500, 300, 300)));
    }

    /// remove 应清理窗口的所有动画状态
    #[test]
    fn test_animation_remove_cleans_up() {
        let config = AnimationConfig {
            enabled: true,
            duration: Duration::from_millis(500),
        };
        let mut mgr = AnimationManager::new(config);

        mgr.set_target(1, 0, 0, 100, 100);
        mgr.set_target(1, 500, 500, 300, 300);
        assert!(mgr.has_active_animations());
        assert!(mgr.get_position(1).is_some());

        mgr.remove(1);

        assert!(!mgr.has_active_animations());
        assert!(mgr.get_position(1).is_none());
    }

    /// 目标位置与当前位置相同时不应启动动画
    #[test]
    fn test_animation_same_position_no_animation() {
        let config = AnimationConfig {
            enabled: true,
            duration: Duration::from_millis(200),
        };
        let mut mgr = AnimationManager::new(config);

        // 初始位置
        mgr.set_target(1, 100, 200, 800, 600);

        // 设置相同的目标位置
        mgr.set_target(1, 100, 200, 800, 600);

        // 不应有活跃动画
        assert!(!mgr.has_active_animations());
        assert_eq!(mgr.get_position(1), Some((100, 200, 800, 600)));
    }
}
