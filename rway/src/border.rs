// border.rs — 窗口边框渲染元素生成
//
// 为每个平铺窗口在其外围生成 4 个 SolidColorRenderElement（上下左右各一条），
// 聚焦窗口使用高亮色，非聚焦窗口使用暗色。

use smithay::{
    backend::renderer::element::{
        solid::{SolidColorBuffer, SolidColorRenderElement},
        Kind,
    },
    utils::{Logical, Physical, Point, Rectangle, Scale, Size},
};

/// 边框样式配置
#[derive(Debug, Clone)]
pub struct BorderConfig {
    /// 边框宽度（逻辑像素）
    pub width: i32,
    /// 聚焦窗口边框颜色 [R, G, B, A]
    pub focused_color: [f32; 4],
    /// 非聚焦窗口边框颜色 [R, G, B, A]
    pub unfocused_color: [f32; 4],
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            width: 2,
            focused_color: [0.3, 0.6, 0.9, 1.0],   // 蓝色（聚焦）
            unfocused_color: [0.3, 0.3, 0.3, 1.0], // 灰色（非聚焦）
        }
    }
}

/// 为一个窗口的逻辑矩形生成 4 条边框渲染元素（上、下、左、右）。
///
/// 边框绘制在窗口 **外侧**，不覆盖窗口内容。
///
/// # 参数
/// - `window_geo` — 窗口在逻辑坐标系下的位置和尺寸
/// - `color`      — 边框颜色 [R, G, B, A]
/// - `border_width` — 边框宽度（逻辑像素）
/// - `scale`      — 输出缩放因子
///
/// # 返回值
/// 包含 4 个 `SolidColorRenderElement` 的向量（顺序：上、下、左、右）。
/// 若 `border_width <= 0`，则返回空向量。
pub fn window_borders(
    window_geo: Rectangle<i32, Logical>,
    color: [f32; 4],
    border_width: i32,
    scale: Scale<f64>,
) -> Vec<SolidColorRenderElement> {
    // 边框宽度非正时不渲染
    if border_width <= 0 {
        return vec![];
    }

    let bw = border_width;
    let x = window_geo.loc.x;
    let y = window_geo.loc.y;
    let w = window_geo.size.w;
    let h = window_geo.size.h;

    // 窗口宽度或高度为非正值时不渲染（防御性检查，同时避免 Smithay Size 对负数 panic）
    if w <= 0 || h <= 0 {
        return vec![];
    }

    // 边框宽度超大时仍需确保 bw 本身为正（已在上方校验 border_width>0 分支保证）

    vec![
        // 上边框：从 (x-bw, y-bw)，宽 w+2*bw，高 bw
        make_border_element(x - bw, y - bw, w + 2 * bw, bw, color, scale),
        // 下边框：从 (x-bw, y+h)，宽 w+2*bw，高 bw
        make_border_element(x - bw, y + h, w + 2 * bw, bw, color, scale),
        // 左边框：从 (x-bw, y)，宽 bw，高 h
        make_border_element(x - bw, y, bw, h, color, scale),
        // 右边框：从 (x+w, y)，宽 bw，高 h
        make_border_element(x + w, y, bw, h, color, scale),
    ]
}

/// 创建单条边框渲染元素（内部辅助函数）。
///
/// 接受逻辑坐标，转换为物理像素后构造 `SolidColorRenderElement`。
fn make_border_element(
    lx: i32,
    ly: i32,
    lw: i32,
    lh: i32,
    color: [f32; 4],
    scale: Scale<f64>,
) -> SolidColorRenderElement {
    // 将逻辑尺寸转换为物理像素（四舍五入）
    let logical_size: Size<i32, Logical> = Size::from((lw, lh));
    let physical_size: Size<i32, Physical> = logical_size.to_physical_precise_round(scale);

    // 将逻辑位置转换为物理像素
    let logical_loc: Point<i32, Logical> = Point::from((lx, ly));
    let physical_loc: Point<i32, Physical> = logical_loc.to_physical_precise_round(scale);

    // 物理坐标矩形（此处由 SolidColorBuffer 内部计算，直接使用 physical_loc）
    let _geometry = Rectangle::new(physical_loc, physical_size);

    // 使用 SolidColorBuffer 辅助创建元素（统一管理 ID 和提交计数）
    let buffer = SolidColorBuffer::new(logical_size, color);
    SolidColorRenderElement::from_buffer(&buffer, physical_loc, scale, 1.0, Kind::Unspecified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::{Logical, Point, Rectangle, Scale, Size};

    // ===== 辅助函数 =====

    /// 构造一个标准测试用窗口矩形
    fn make_window(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Logical> {
        Rectangle::new(Point::from((x, y)), Size::from((w, h)))
    }

    /// 1x 缩放（无缩放）
    fn scale_1x() -> Scale<f64> {
        Scale::from(1.0)
    }

    // ===== BorderConfig 测试 =====

    /// 默认配置应包含合理的边框宽度
    #[test]
    fn border_config_default_width_is_2() {
        let cfg = BorderConfig::default();
        assert_eq!(cfg.width, 2);
    }

    /// 默认聚焦颜色 alpha 应为 1.0（完全不透明）
    #[test]
    fn border_config_focused_color_is_opaque() {
        let cfg = BorderConfig::default();
        assert_eq!(cfg.focused_color[3], 1.0);
    }

    /// 默认非聚焦颜色 alpha 应为 1.0（完全不透明）
    #[test]
    fn border_config_unfocused_color_is_opaque() {
        let cfg = BorderConfig::default();
        assert_eq!(cfg.unfocused_color[3], 1.0);
    }

    /// 聚焦色与非聚焦色应不同（视觉可区分）
    #[test]
    fn border_config_focused_and_unfocused_colors_differ() {
        let cfg = BorderConfig::default();
        assert_ne!(cfg.focused_color, cfg.unfocused_color);
    }

    // ===== window_borders 正常路径测试 =====

    /// 正常窗口应生成恰好 4 个边框元素
    #[test]
    fn window_borders_returns_four_elements_for_normal_window() {
        let geo = make_window(100, 100, 800, 600);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert_eq!(result.len(), 4, "应生成 4 条边框");
    }

    /// 边框宽度为 1 时也应生成 4 个元素
    #[test]
    fn window_borders_works_with_border_width_1() {
        let geo = make_window(0, 0, 400, 300);
        let color = [0.3, 0.3, 0.3, 1.0];
        let result = window_borders(geo, color, 1, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 位于原点的窗口也应正常生成 4 条边框
    #[test]
    fn window_borders_works_at_origin() {
        let geo = make_window(0, 0, 200, 150);
        let color = [1.0, 0.0, 0.0, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 2x 高分辨率缩放时应生成 4 个元素
    #[test]
    fn window_borders_works_with_2x_scale() {
        let geo = make_window(50, 50, 600, 400);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, Scale::from(2.0));
        assert_eq!(result.len(), 4);
    }

    /// 1.5x 非整数缩放时应生成 4 个元素
    #[test]
    fn window_borders_works_with_fractional_scale() {
        let geo = make_window(0, 0, 1000, 800);
        let color = [0.5, 0.5, 0.5, 1.0];
        let result = window_borders(geo, color, 3, Scale::from(1.5));
        assert_eq!(result.len(), 4);
    }

    // ===== 边界条件 / 边缘情况测试 =====

    /// 边框宽度为 0 时应返回空向量
    #[test]
    fn window_borders_returns_empty_for_zero_border_width() {
        let geo = make_window(100, 100, 800, 600);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 0, scale_1x());
        assert!(result.is_empty(), "边框宽度为 0 时不应生成任何元素");
    }

    /// 边框宽度为负数时应返回空向量
    #[test]
    fn window_borders_returns_empty_for_negative_border_width() {
        let geo = make_window(100, 100, 800, 600);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, -1, scale_1x());
        assert!(result.is_empty(), "边框宽度为负数时不应生成任何元素");
    }

    /// 窗口宽度为 0 时应返回空向量（无效窗口）
    #[test]
    fn window_borders_returns_empty_for_zero_window_width() {
        let geo = make_window(100, 100, 0, 600);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert!(result.is_empty(), "窗口宽度为 0 时不应生成边框");
    }

    /// 窗口高度为 0 时应返回空向量（无效窗口）
    #[test]
    fn window_borders_returns_empty_for_zero_window_height() {
        let geo = make_window(100, 100, 800, 0);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert!(result.is_empty(), "窗口高度为 0 时不应生成边框");
    }

    /// 窗口宽高均为负数时，Smithay 的 Size debug_assert 会 panic（在 release 中不触发）。
    /// 为验证我们的防御性分支，用 catch_unwind 检测确实触发了 panic 或返回空向量。
    #[test]
    fn window_borders_handles_negative_window_size_defensively() {
        // Smithay Size::from 在 debug 模式下对负尺寸会 panic（debug_assert）。
        // 我们使用 catch_unwind 来验证代码在 debug 构建中的行为是安全的
        // （要么 panic，要么返回空），而不是静默地产生错误数据。
        let result = std::panic::catch_unwind(|| {
            let geo = make_window(100, 100, -100, -100);
            window_borders(geo, [0.3, 0.6, 0.9, 1.0], 2, scale_1x())
        });
        // 在 debug 模式下应 panic（Smithay debug_assert），在 release 模式下应返回空向量
        match result {
            Err(_) => { /* debug 模式下的预期 panic —— 行为符合预期 */ }
            Ok(borders) => {
                assert!(borders.is_empty(), "release 模式下，负尺寸窗口不应生成边框");
            }
        }
    }

    /// 极大窗口（4K 屏）应正常生成 4 条边框
    #[test]
    fn window_borders_works_with_large_window() {
        let geo = make_window(0, 0, 3840, 2160);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 极小窗口（1x1 像素）应正常生成 4 条边框
    #[test]
    fn window_borders_works_with_minimal_window() {
        let geo = make_window(0, 0, 1, 1);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 窗口位于负坐标处（屏幕外）也应正常生成 4 条边框
    #[test]
    fn window_borders_works_with_negative_window_position() {
        let geo = make_window(-50, -50, 200, 150);
        let color = [0.3, 0.6, 0.9, 1.0];
        let result = window_borders(geo, color, 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 极大边框宽度（大于窗口尺寸）应仍然生成 4 条边框
    #[test]
    fn window_borders_works_with_very_large_border_width() {
        let geo = make_window(0, 0, 100, 100);
        let color = [0.3, 0.6, 0.9, 1.0];
        // 边框宽度大于窗口宽高，但仍应生成（视觉上会重叠，但不是函数的职责阻止）
        let result = window_borders(geo, color, 200, scale_1x());
        assert_eq!(result.len(), 4);
    }

    // ===== BorderConfig 与 window_borders 联合测试 =====

    /// 使用默认配置的聚焦色应生成 4 条边框
    #[test]
    fn window_borders_with_default_focused_color() {
        let cfg = BorderConfig::default();
        let geo = make_window(100, 200, 640, 480);
        let result = window_borders(geo, cfg.focused_color, cfg.width, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 使用默认配置的非聚焦色应生成 4 条边框
    #[test]
    fn window_borders_with_default_unfocused_color() {
        let cfg = BorderConfig::default();
        let geo = make_window(100, 200, 640, 480);
        let result = window_borders(geo, cfg.unfocused_color, cfg.width, scale_1x());
        assert_eq!(result.len(), 4);
    }

    /// 全透明颜色（alpha=0）也应生成 4 条边框（透明但存在）
    #[test]
    fn window_borders_with_fully_transparent_color() {
        let geo = make_window(0, 0, 400, 300);
        let transparent = [0.0, 0.0, 0.0, 0.0];
        let result = window_borders(geo, transparent, 2, scale_1x());
        assert_eq!(result.len(), 4);
    }
}
