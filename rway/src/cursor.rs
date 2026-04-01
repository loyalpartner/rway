// cursor.rs -- 光标渲染元素生成
//
// 当前使用白色小方块作为光标占位符。
// 后续可替换为 xcursor 主题渲染或客户端 surface 光标。

use smithay::{
    backend::renderer::element::{
        solid::{SolidColorBuffer, SolidColorRenderElement},
        Kind,
    },
    utils::{Logical, Physical, Point, Scale, Size},
};

/// 光标方块的尺寸（逻辑像素）
const CURSOR_SIZE: i32 = 8;

/// 光标颜色（白色，完全不透明）
const CURSOR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// 在给定指针位置生成一个白色小方块作为光标渲染元素。
///
/// # 参数
/// - `pos` -- 指针当前逻辑坐标
/// - `scale` -- 输出缩放因子
pub fn cursor_square_element(
    pos: Point<f64, Logical>,
    scale: Scale<f64>,
) -> SolidColorRenderElement {
    let logical_size: Size<i32, Logical> = Size::from((CURSOR_SIZE, CURSOR_SIZE));
    let logical_loc: Point<i32, Logical> = Point::from((pos.x as i32, pos.y as i32));
    let physical_loc: Point<i32, Physical> = logical_loc.to_physical_precise_round(scale);

    let buffer = SolidColorBuffer::new(logical_size, CURSOR_COLOR);
    SolidColorRenderElement::from_buffer(&buffer, physical_loc, scale, 1.0, Kind::Cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::{Logical, Point, Scale};

    #[test]
    fn cursor_element_is_created_at_origin() {
        let pos: Point<f64, Logical> = Point::from((0.0, 0.0));
        let _element = cursor_square_element(pos, Scale::from(1.0));
    }

    #[test]
    fn cursor_element_is_created_at_arbitrary_position() {
        let pos: Point<f64, Logical> = Point::from((500.5, 300.7));
        let _element = cursor_square_element(pos, Scale::from(1.0));
    }

    #[test]
    fn cursor_element_works_with_2x_scale() {
        let pos: Point<f64, Logical> = Point::from((100.0, 200.0));
        let _element = cursor_square_element(pos, Scale::from(2.0));
    }

    #[test]
    fn cursor_size_is_positive() {
        assert!(CURSOR_SIZE > 0);
    }

    #[test]
    fn cursor_color_is_opaque() {
        assert_eq!(CURSOR_COLOR[3], 1.0);
    }
}
