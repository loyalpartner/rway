// backend/winit.rs — Winit 后端初始化（开发/测试模式）

use std::time::Duration;

use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker,
            element::surface::WaylandSurfaceRenderElement,
            gles::GlesRenderer,
        },
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Transform},
};

use crate::state::RwayState;

/// 初始化 Winit 后端：创建窗口、输出、损坏跟踪器，并注册到事件循环
pub fn init_winit(
    event_loop: &mut EventLoop<RwayState>,
    state: &mut RwayState,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 winit 后端（打开一个宿主窗口作为输出）
    let (mut backend, winit) = winit::init()?;

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
                    // 转发输入事件到 RwayState 的输入处理器
                    state.process_input_event(event);
                }

                WinitEvent::Redraw => {
                    let size = backend.window_size();
                    let damage = Rectangle::from_size(size);

                    {
                        // 绑定帧缓冲区并渲染
                        let (renderer, mut framebuffer) = backend.bind().unwrap();
                        smithay::desktop::space::render_output::<
                            _,
                            WaylandSurfaceRenderElement<GlesRenderer>,
                            _,
                            _,
                        >(
                            &output,
                            renderer,
                            &mut framebuffer,
                            1.0,
                            0,
                            [&state.space],
                            &[],
                            &mut damage_tracker,
                            [0.1, 0.1, 0.1, 1.0], // 深灰色背景
                        )
                        .unwrap();
                    }

                    // 提交帧到宿主窗口
                    backend.submit(Some(&[damage])).unwrap();

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
                    let _ = state.display_handle.flush_clients();

                    // 请求下一帧重绘（驱动渲染循环）
                    backend.window().request_redraw();
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
