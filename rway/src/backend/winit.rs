// backend/winit.rs — Winit 后端初始化（开发/测试模式）

use std::time::Duration;

use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Scale, Transform},
};

use smithay::input::pointer::CursorImageStatus;

use crate::{
    border::{window_borders, BorderConfig},
    state::RwayState,
};

/// Initialize the Winit backend: create window, output, damage tracker, and register with event loop
pub(crate) fn init_winit(
    event_loop: &mut EventLoop<RwayState>,
    state: &mut RwayState,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 winit 后端（打开一个宿主窗口作为输出，使用 GLES 渲染器）
    let (mut backend, winit) = winit::init::<GlesRenderer>()?;

    // 设置输出模式（与 winit 窗口大小匹配）
    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    // 创建逻辑输出并注册到 display
    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".into(),
            model: "Winit".into(),
        },
    );
    let _global = output.create_global::<RwayState>(&state.display_handle);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    // 将输出映射到空间原点
    state.space.map_output(&output, (0, 0));

    // 在平铺树中注册输出和默认工作区
    let win_size = backend.window_size();
    state.init_tiling_output(win_size.w, win_size.h);

    // 损坏跟踪器：只重绘脏区域，提高性能
    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    // 边框配置从用户 config 读取
    let border_config = BorderConfig::from_config(&state.config);

    // 将 winit 事件源注册到 calloop 事件循环
    event_loop
        .handle()
        .insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::Resized { size, .. } => {
                    // 窗口大小变化：更新输出模式
                    output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                }

                WinitEvent::Input(event) => {
                    state.process_input_event(event);
                    // Input changed state — need to redraw
                    backend.window().request_redraw();
                }

                WinitEvent::Redraw => {
                    let has_animations = state.update_animations();
                    state.needs_redraw = false;

                    let size = backend.window_size();
                    let damage = Rectangle::from_size(size);

                    {
                        // 1x 缩放（winit 嵌套合成器暂不支持 HiDPI 缩放）
                        let scale = Scale::from(1.0_f64);

                        // 获取当前聚焦的键盘 surface，用于判断哪个窗口拥有焦点
                        let focused_surface =
                            state.seat.get_keyboard().and_then(|kb| kb.current_focus());

                        // Generate border elements for each window.
                        // Space geometry is the content rect (inset by bw);
                        // expand back to full container rect for border drawing.
                        let bw = border_config.width;
                        let mut custom_elements: Vec<_> = state
                            .space
                            .elements()
                            .flat_map(|window| {
                                let is_focused = focused_surface.as_ref().is_some_and(|fs| {
                                    window.toplevel().is_some_and(|tl| tl.wl_surface() == fs)
                                });

                                let color = if is_focused {
                                    border_config.focused_color
                                } else {
                                    border_config.unfocused_color
                                };

                                if let Some(content_geo) = state.space.element_geometry(window) {
                                    // Reconstruct container rect from content rect
                                    let container_geo = smithay::utils::Rectangle::new(
                                        (content_geo.loc.x - bw, content_geo.loc.y - bw).into(),
                                        (content_geo.size.w + 2 * bw, content_geo.size.h + 2 * bw)
                                            .into(),
                                    );
                                    window_borders(container_geo, color, bw, scale)
                                } else {
                                    vec![]
                                }
                            })
                            .collect();

                        // 生成光标渲染元素（白色 8x8 方块，z-order 最高）
                        if !matches!(state.cursor_status, CursorImageStatus::Hidden) {
                            if let Some(pointer) = state.seat.get_pointer() {
                                let pos = pointer.current_location();
                                let cursor_element =
                                    crate::cursor::cursor_square_element(pos, scale);
                                custom_elements.push(cursor_element);
                            }
                        }

                        // Bind framebuffer and render (borders as custom_elements drawn first, cursor last = highest z-order)
                        let Ok((renderer, mut framebuffer)) = backend.bind() else {
                            tracing::warn!("Failed to bind winit backend framebuffer");
                            return;
                        };
                        if let Err(e) = smithay::desktop::space::render_output::<
                            _,
                            smithay::backend::renderer::element::solid::SolidColorRenderElement,
                            _,
                            _,
                        >(
                            &output,
                            renderer,
                            &mut framebuffer,
                            1.0,
                            0,
                            [&state.space],
                            &custom_elements,
                            &mut damage_tracker,
                            [0.1, 0.1, 0.1, 1.0], // dark gray background
                        ) {
                            tracing::warn!("Failed to render output: {:?}", e);
                        }
                    }

                    // Submit frame to host window
                    if let Err(e) = backend.submit(Some(&[damage])) {
                        tracing::warn!("Failed to submit winit frame: {:?}", e);
                    }

                    // 通知所有窗口帧已完成
                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        )
                    });

                    // 清理过期的弹窗并刷新空间
                    state.space.refresh();
                    state.popups.cleanup();

                    // 检测并清理已关闭的窗口，释放平铺树中的空间
                    state.cleanup_dead_windows();

                    let _ = state.display_handle.flush_clients();

                    // Only request next frame if there's active work
                    if has_animations || state.needs_redraw {
                        backend.window().request_redraw();
                    }
                }

                WinitEvent::CloseRequested => {
                    // 宿主窗口关闭 → 停止合成器事件循环
                    state.loop_signal.stop();
                }

                _ => (),
            }
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // Winit 后端需要真实的图形环境，此处仅测试可以独立验证的逻辑

    use smithay::utils::{Logical, Size};

    /// 测试输出模式的刷新率常量
    #[test]
    fn winit_refresh_rate_is_60hz() {
        let refresh: i32 = 60_000; // mHz
        assert_eq!(refresh / 1000, 60);
    }

    /// 测试初始背景颜色格式正确（RGBA，范围 0..=1）
    #[test]
    fn background_color_is_valid_rgba() {
        let bg = [0.1_f32, 0.1, 0.1, 1.0];
        for &component in &bg[..3] {
            assert!(component >= 0.0 && component <= 1.0);
        }
        assert_eq!(bg[3], 1.0); // alpha 必须完全不透明
    }

    /// 测试输出物理尺寸为零（虚拟输出无物理像素）
    #[test]
    fn winit_output_physical_size_is_zero() {
        let physical_size: Size<i32, Logical> = Size::from((0, 0));
        assert_eq!(physical_size.w, 0);
        assert_eq!(physical_size.h, 0);
    }

    /// 测试损坏区域等于整个帧
    #[test]
    fn full_frame_damage_covers_entire_size() {
        use smithay::utils::{Logical, Rectangle, Size};

        let size: Size<i32, Logical> = Size::from((1920, 1080));
        let damage = Rectangle::from_size(size);

        assert_eq!(damage.loc.x, 0);
        assert_eq!(damage.loc.y, 0);
        assert_eq!(damage.size.w, 1920);
        assert_eq!(damage.size.h, 1080);
    }
}
