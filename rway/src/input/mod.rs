// input/mod.rs — 输入事件处理（键盘、指针、触摸）

pub(crate) mod keybindings;

use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent,
    },
    input::{
        keyboard::FilterResult,
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
};

use smithay::wayland::seat::WaylandFocus;

use crate::state::RwayState;

impl RwayState {
    /// 处理输入后端发来的各类输入事件
    pub fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event, .. } => {
                let serial = SERIAL_COUNTER.next_serial();
                let time = Event::time_msec(&event);

                // 克隆 keybindings 引用以避免借用冲突
                let bindings = self.config.keybindings.clone();

                let Some(keyboard) = self.seat.get_keyboard() else {
                    return;
                };
                keyboard.input::<(), _>(
                    self,
                    event.key_code(),
                    event.state(),
                    serial,
                    time,
                    |state_ref, modifiers, keysym_handle| {
                        if event.state() == KeyState::Pressed {
                            let raw_sym = keysym_handle.raw_latin_sym_or_raw_current_sym();
                            let modified_sym = keysym_handle.modified_sym();
                            tracing::debug!(
                                "按键: raw={:?} modified={:?} mods=[logo={} alt={} shift={} ctrl={}]",
                                raw_sym.map(|s| s.raw()),
                                modified_sym.raw(),
                                modifiers.logo, modifiers.alt, modifiers.shift, modifiers.ctrl,
                            );
                            if let Some(raw_sym) = raw_sym {
                                let sym_val = raw_sym.raw();
                                let key_name = keybindings::keysym_to_name(sym_val);
                                tracing::debug!("  -> keysym={:#x} name={:?}", sym_val, key_name);
                                if let Some(action) = keybindings::find_matching_binding(
                                    &bindings,
                                    modifiers,
                                    sym_val,
                                ) {
                                    tracing::info!("快捷键匹配: {:?}", action);
                                    let action = action.clone();
                                    keybindings::execute_action(state_ref, &action);
                                    return FilterResult::Intercept(());
                                }
                            }
                        }
                        FilterResult::Forward
                    },
                );
            }

            InputEvent::PointerMotion { event, .. } => {
                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                let current = pointer.current_location();

                // 相对移动：累加到当前位置，并 clamp 到输出范围
                let mut new_pos = current + event.delta();

                // 限制在所有输出的联合几何范围内
                if let Some(output) = self.space.outputs().next() {
                    if let Some(geo) = self.space.output_geometry(output) {
                        new_pos.x = new_pos
                            .x
                            .clamp(geo.loc.x as f64, (geo.loc.x + geo.size.w) as f64 - 1.0);
                        new_pos.y = new_pos
                            .y
                            .clamp(geo.loc.y as f64, (geo.loc.y + geo.size.h) as f64 - 1.0);
                    }
                }

                let serial = SERIAL_COUNTER.next_serial();
                let under = self.surface_under(new_pos);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: new_pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }

            InputEvent::PointerMotionAbsolute { event, .. } => {
                let Some(output) = self.space.outputs().next() else {
                    return;
                };
                let Some(output_geo) = self.space.output_geometry(output) else {
                    return;
                };

                // 将绝对坐标变换到逻辑坐标系
                let pos = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();

                let serial = SERIAL_COUNTER.next_serial();
                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                let under = self.surface_under(pos);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }

            InputEvent::PointerButton { event, .. } => {
                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                let Some(keyboard) = self.seat.get_keyboard() else {
                    return;
                };

                let serial = SERIAL_COUNTER.next_serial();
                let button = event.button_code();
                let button_state = event.state();

                if ButtonState::Pressed == button_state && !pointer.is_grabbed() {
                    if let Some((window, _loc)) = self
                        .space
                        .element_under(pointer.current_location())
                        .map(|(w, l)| (w.clone(), l))
                    {
                        self.space.raise_element(&window, true);
                        let focus_surface = window
                            .toplevel()
                            .map(|t| t.wl_surface().clone())
                            .or_else(|| window.wl_surface().map(|s| s.into_owned()));
                        keyboard.set_focus(self, focus_surface, serial);

                        // Sync tiling tree focused_child path to this window
                        if let Some((&win_id, _)) = self.window_map.iter().find(|(_, w)| {
                            w.toplevel()
                                .zip(window.toplevel())
                                .map_or(false, |(a, b)| a.wl_surface() == b.wl_surface())
                        }) {
                            self.tiling.focus_window(win_id);
                        }
                        self.space.elements().for_each(|w| {
                            if let Some(t) = w.toplevel() {
                                t.send_pending_configure();
                            }
                        });
                    } else {
                        // 点击空白处：取消所有窗口激活状态
                        self.space.elements().for_each(|w| {
                            w.set_activated(false);
                            if let Some(t) = w.toplevel() {
                                t.send_pending_configure();
                            }
                        });
                        keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                    }
                }

                pointer.button(
                    self,
                    &ButtonEvent {
                        button,
                        state: button_state,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }

            InputEvent::PointerAxis { event, .. } => {
                let source = event.source();

                let horizontal_amount = event.amount(Axis::Horizontal).unwrap_or_else(|| {
                    event.amount_v120(Axis::Horizontal).unwrap_or(0.0) * 15.0 / 120.0
                });
                let vertical_amount = event.amount(Axis::Vertical).unwrap_or_else(|| {
                    event.amount_v120(Axis::Vertical).unwrap_or(0.0) * 15.0 / 120.0
                });
                let horizontal_amount_discrete = event.amount_v120(Axis::Horizontal);
                let vertical_amount_discrete = event.amount_v120(Axis::Vertical);

                let mut frame = AxisFrame::new(event.time_msec()).source(source);

                if horizontal_amount != 0.0 {
                    frame = frame.value(Axis::Horizontal, horizontal_amount);
                    if let Some(discrete) = horizontal_amount_discrete {
                        frame = frame.v120(Axis::Horizontal, discrete as i32);
                    }
                }
                if vertical_amount != 0.0 {
                    frame = frame.value(Axis::Vertical, vertical_amount);
                    if let Some(discrete) = vertical_amount_discrete {
                        frame = frame.v120(Axis::Vertical, discrete as i32);
                    }
                }

                // 手指滑动手势：如果轴值为 0 则发送 stop 事件
                if source == AxisSource::Finger {
                    if event.amount(Axis::Horizontal) == Some(0.0) {
                        frame = frame.stop(Axis::Horizontal);
                    }
                    if event.amount(Axis::Vertical) == Some(0.0) {
                        frame = frame.stop(Axis::Vertical);
                    }
                }

                let Some(pointer) = self.seat.get_pointer() else {
                    return;
                };
                pointer.axis(self, frame);
                pointer.frame(self);
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    // 测试输入处理中的纯数学逻辑

    use smithay::utils::{Logical, Point, Size};

    /// 测试绝对坐标变换：将归一化坐标变换到输出几何
    ///
    /// position_transformed(output_size) + output_loc 模拟
    #[test]
    fn absolute_position_transform_basic() {
        // 模拟 1920x1080 的输出，位于 (0,0)
        let output_size: Size<i32, Logical> = Size::from((1920, 1080));
        let output_loc: Point<f64, Logical> = Point::from((0.0, 0.0));

        // 归一化位置 (0.5, 0.5) = 屏幕中央
        let normalized = Point::<f64, Logical>::from((0.5, 0.5));
        let transformed = Point::from((
            normalized.x * output_size.w as f64,
            normalized.y * output_size.h as f64,
        )) + output_loc;

        assert_eq!(transformed.x, 960.0);
        assert_eq!(transformed.y, 540.0);
    }

    /// 测试坐标变换到非零偏移的输出
    #[test]
    fn absolute_position_transform_with_offset() {
        let output_size: Size<i32, Logical> = Size::from((1920, 1080));
        let output_loc: Point<f64, Logical> = Point::from((1920.0, 0.0)); // 第二块屏幕

        let normalized = Point::<f64, Logical>::from((0.0, 0.0)); // 左上角
        let transformed = Point::from((
            normalized.x * output_size.w as f64,
            normalized.y * output_size.h as f64,
        )) + output_loc;

        assert_eq!(transformed.x, 1920.0);
        assert_eq!(transformed.y, 0.0);
    }

    /// 测试坐标变换边界值：右下角
    #[test]
    fn absolute_position_transform_bottom_right() {
        let output_size: Size<i32, Logical> = Size::from((1920, 1080));
        let output_loc: Point<f64, Logical> = Point::from((0.0, 0.0));

        let normalized = Point::<f64, Logical>::from((1.0, 1.0));
        let transformed = Point::from((
            normalized.x * output_size.w as f64,
            normalized.y * output_size.h as f64,
        )) + output_loc;

        assert_eq!(transformed.x, 1920.0);
        assert_eq!(transformed.y, 1080.0);
    }

    /// 测试滚轮轴量计算：v120 到像素量的转换
    #[test]
    fn axis_v120_to_pixel_conversion() {
        // v120 = 120 表示一个"档位"，对应 15 像素（Smithay 惯例）
        let v120 = 120.0_f64;
        let pixels = v120 * 15.0 / 120.0;
        assert_eq!(pixels, 15.0);
    }

    /// 测试 v120 = 240（两档）
    #[test]
    fn axis_v120_two_notches() {
        let v120 = 240.0_f64;
        let pixels = v120 * 15.0 / 120.0;
        assert_eq!(pixels, 30.0);
    }

    /// 测试 v120 = 0（不产生移动）
    #[test]
    fn axis_v120_zero_produces_zero_pixels() {
        let v120 = 0.0_f64;
        let pixels = v120 * 15.0 / 120.0;
        assert_eq!(pixels, 0.0);
    }

    /// 测试负向滚动
    #[test]
    fn axis_v120_negative_direction() {
        let v120 = -120.0_f64;
        let pixels = v120 * 15.0 / 120.0;
        assert_eq!(pixels, -15.0);
    }
}
