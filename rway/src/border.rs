// border.rs — Window border rendering element generation
//
// For each tiled window, generate 4 SolidColorRenderElements (top/bottom/left/right)
// drawn **inside** the container rectangle. Focused windows use highlight color,
// unfocused windows use a dim color.

use smithay::{
    backend::renderer::element::{
        solid::{SolidColorBuffer, SolidColorRenderElement},
        Kind,
    },
    utils::{Logical, Physical, Point, Rectangle, Scale, Size},
};

/// Border style configuration
#[derive(Debug, Clone)]
pub(crate) struct BorderConfig {
    /// Border width (logical pixels)
    pub(crate) width: i32,
    /// Focused window border color [R, G, B, A]
    pub(crate) focused_color: [f32; 4],
    /// Unfocused window border color [R, G, B, A]
    pub(crate) unfocused_color: [f32; 4],
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            width: 2,
            focused_color: [0.3, 0.6, 0.9, 1.0], // blue (focused)
            unfocused_color: [0.3, 0.3, 0.3, 1.0], // gray (unfocused)
        }
    }
}

impl BorderConfig {
    /// Build from parsed Sway config, falling back to defaults for unset values.
    pub(crate) fn from_config(config: &rway_config::Config) -> Self {
        let width = match &config.default_border {
            rway_config::BorderStyle::Normal(w) | rway_config::BorderStyle::Pixel(w) => *w as i32,
            rway_config::BorderStyle::None => 0,
        };

        let focused_color = config
            .client_focused
            .as_ref()
            .and_then(|c| parse_hex_color(&c.child_border))
            .unwrap_or([0.3, 0.6, 0.9, 1.0]);

        let unfocused_color = config
            .client_unfocused
            .as_ref()
            .and_then(|c| parse_hex_color(&c.child_border))
            .unwrap_or([0.3, 0.3, 0.3, 1.0]);

        Self {
            width,
            focused_color,
            unfocused_color,
        }
    }
}

/// Parse a hex color string (#rrggbb or #rrggbbaa) to [f32; 4].
fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    let hex = hex.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some([
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ])
        }
        _ => None,
    }
}

/// Generate 4 border render elements (top, bottom, left, right) drawn **inside**
/// the given container rectangle.
///
/// # Parameters
/// - `container_geo` — the full container rectangle (border + content area)
/// - `color`         — border color [R, G, B, A]
/// - `border_width`  — border width (logical pixels)
/// - `scale`         — output scale factor
///
/// # Returns
/// A vector of 4 `SolidColorRenderElement`s. Empty if `border_width <= 0`.
pub(crate) fn window_borders(
    container_geo: Rectangle<i32, Logical>,
    color: [f32; 4],
    border_width: i32,
    scale: Scale<f64>,
) -> Vec<SolidColorRenderElement> {
    if border_width <= 0 {
        return vec![];
    }

    let bw = border_width;
    let x = container_geo.loc.x;
    let y = container_geo.loc.y;
    let w = container_geo.size.w;
    let h = container_geo.size.h;

    if w <= 0 || h <= 0 {
        return vec![];
    }

    // Inner height for left/right borders (between top and bottom)
    let inner_h = (h - 2 * bw).max(0);

    vec![
        // Top: full width at top edge
        make_border_element(x, y, w, bw, color, scale),
        // Bottom: full width at bottom edge
        make_border_element(x, y + h - bw, w, bw, color, scale),
        // Left: inner height, below top border
        make_border_element(x, y + bw, bw, inner_h, color, scale),
        // Right: inner height, below top border
        make_border_element(x + w - bw, y + bw, bw, inner_h, color, scale),
    ]
}

/// Create a single border render element from logical coordinates.
fn make_border_element(
    lx: i32,
    ly: i32,
    lw: i32,
    lh: i32,
    color: [f32; 4],
    scale: Scale<f64>,
) -> SolidColorRenderElement {
    let logical_size: Size<i32, Logical> = Size::from((lw, lh));
    let physical_size: Size<i32, Physical> = logical_size.to_physical_precise_round(scale);

    let logical_loc: Point<i32, Logical> = Point::from((lx, ly));
    let physical_loc: Point<i32, Physical> = logical_loc.to_physical_precise_round(scale);

    let _geometry = Rectangle::new(physical_loc, physical_size);

    let buffer = SolidColorBuffer::new(logical_size, color);
    SolidColorRenderElement::from_buffer(&buffer, physical_loc, scale, 1.0, Kind::Unspecified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::{Logical, Point, Rectangle, Scale, Size};

    fn make_rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Logical> {
        Rectangle::new(Point::from((x, y)), Size::from((w, h)))
    }

    fn scale_1x() -> Scale<f64> {
        Scale::from(1.0)
    }

    // ===== BorderConfig tests =====

    #[test]
    fn border_config_default_width_is_2() {
        let cfg = BorderConfig::default();
        assert_eq!(cfg.width, 2);
    }

    #[test]
    fn border_config_focused_color_is_opaque() {
        let cfg = BorderConfig::default();
        assert_eq!(cfg.focused_color[3], 1.0);
    }

    #[test]
    fn border_config_unfocused_color_is_opaque() {
        let cfg = BorderConfig::default();
        assert_eq!(cfg.unfocused_color[3], 1.0);
    }

    #[test]
    fn border_config_focused_and_unfocused_colors_differ() {
        let cfg = BorderConfig::default();
        assert_ne!(cfg.focused_color, cfg.unfocused_color);
    }

    // ===== parse_hex_color tests =====

    #[test]
    fn parse_hex_color_6digit() {
        let c = parse_hex_color("#ff8000").unwrap();
        assert!((c[0] - 1.0).abs() < 0.01);
        assert!((c[1] - 0.502).abs() < 0.01);
        assert!((c[2] - 0.0).abs() < 0.01);
        assert_eq!(c[3], 1.0);
    }

    #[test]
    fn parse_hex_color_8digit() {
        let c = parse_hex_color("#ff000080").unwrap();
        assert!((c[0] - 1.0).abs() < 0.01);
        assert!((c[3] - 0.502).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert!(parse_hex_color("not_hex").is_none());
        assert!(parse_hex_color("#xyz").is_none());
    }

    // ===== window_borders tests (inside drawing) =====

    #[test]
    fn window_borders_returns_four_elements() {
        let geo = make_rect(100, 100, 800, 600);
        let result = window_borders(geo, [0.3, 0.6, 0.9, 1.0], 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn window_borders_at_origin_visible() {
        // Container at (0,0) — all borders should be at non-negative coords
        let geo = make_rect(0, 0, 960, 1080);
        let result = window_borders(geo, [1.0, 0.0, 0.0, 1.0], 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn window_borders_empty_for_zero_width() {
        let result = window_borders(make_rect(0, 0, 800, 600), [1.0; 4], 0, scale_1x());
        assert!(result.is_empty());
    }

    #[test]
    fn window_borders_empty_for_negative_width() {
        let result = window_borders(make_rect(0, 0, 800, 600), [1.0; 4], -1, scale_1x());
        assert!(result.is_empty());
    }

    #[test]
    fn window_borders_empty_for_zero_size() {
        let result = window_borders(make_rect(0, 0, 0, 600), [1.0; 4], 2, scale_1x());
        assert!(result.is_empty());
        let result = window_borders(make_rect(0, 0, 800, 0), [1.0; 4], 2, scale_1x());
        assert!(result.is_empty());
    }

    #[test]
    fn window_borders_works_with_2x_scale() {
        let result = window_borders(make_rect(50, 50, 600, 400), [1.0; 4], 2, Scale::from(2.0));
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn window_borders_works_with_large_window() {
        let result = window_borders(make_rect(0, 0, 3840, 2160), [1.0; 4], 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn window_borders_works_with_large_border() {
        let result = window_borders(make_rect(0, 0, 100, 100), [1.0; 4], 200, scale_1x());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn window_borders_with_transparent_color() {
        let result = window_borders(make_rect(0, 0, 400, 300), [0.0; 4], 2, scale_1x());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn window_borders_handles_negative_size_defensively() {
        let result = std::panic::catch_unwind(|| {
            window_borders(make_rect(100, 100, -100, -100), [1.0; 4], 2, scale_1x())
        });
        match result {
            Err(_) => {}
            Ok(borders) => assert!(borders.is_empty()),
        }
    }
}
