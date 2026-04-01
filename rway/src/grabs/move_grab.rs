// grabs/move_grab.rs — 窗口移动抓取
//
// 用户拖拽窗口标题栏时，合成器进入 MoveSurfaceGrab 状态。

use crate::state::RwayState;
use smithay::{
    desktop::Window,
    input::pointer::{
        AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
        GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
        GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
        GrabStartData as PointerGrabStartData, MotionEvent, PointerGrab, PointerInnerHandle,
        RelativeMotionEvent,
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

pub struct MoveSurfaceGrab {
    pub start_data: PointerGrabStartData<RwayState>,
    pub window: Window,
    pub initial_window_location: Point<i32, Logical>,
}

impl PointerGrab<RwayState> for MoveSurfaceGrab {
    fn motion(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        _focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &MotionEvent,
    ) {
        // 抓取期间不给任何客户端指针焦点
        handle.motion(data, None, event);

        let delta = event.location - self.start_data.location;
        let new_location = self.initial_window_location.to_f64() + delta;
        data.space
            .map_element(self.window.clone(), new_location.to_i32_round(), true);
    }

    fn relative_motion(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        handle.relative_motion(data, focus, event);
    }

    fn button(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);

        // BTN_LEFT（linux/input-event-codes.h）
        const BTN_LEFT: u32 = 0x110;

        if !handle.current_pressed().contains(&BTN_LEFT) {
            // 没有按键按下，释放抓取
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn axis(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        details: AxisFrame,
    ) {
        handle.axis(data, details);
    }

    fn frame(&mut self, data: &mut RwayState, handle: &mut PointerInnerHandle<'_, RwayState>) {
        handle.frame(data);
    }

    fn gesture_swipe_begin(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GestureSwipeBeginEvent,
    ) {
        handle.gesture_swipe_begin(data, event);
    }

    fn gesture_swipe_update(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GestureSwipeUpdateEvent,
    ) {
        handle.gesture_swipe_update(data, event);
    }

    fn gesture_swipe_end(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GestureSwipeEndEvent,
    ) {
        handle.gesture_swipe_end(data, event);
    }

    fn gesture_pinch_begin(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GesturePinchBeginEvent,
    ) {
        handle.gesture_pinch_begin(data, event);
    }

    fn gesture_pinch_update(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GesturePinchUpdateEvent,
    ) {
        handle.gesture_pinch_update(data, event);
    }

    fn gesture_pinch_end(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GesturePinchEndEvent,
    ) {
        handle.gesture_pinch_end(data, event);
    }

    fn gesture_hold_begin(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GestureHoldBeginEvent,
    ) {
        handle.gesture_hold_begin(data, event);
    }

    fn gesture_hold_end(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        event: &GestureHoldEndEvent,
    ) {
        handle.gesture_hold_end(data, event);
    }

    fn start_data(&self) -> &PointerGrabStartData<RwayState> {
        &self.start_data
    }

    fn unset(&mut self, _data: &mut RwayState) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::Point;

    /// 测试 MoveSurfaceGrab 位移计算：新位置 = 初始位置 + delta
    #[test]
    fn move_grab_delta_calculation() {
        // 验证位移计算逻辑（纯数学，不依赖 Wayland）
        let initial: Point<i32, Logical> = Point::from((100, 200));
        let start_location: Point<f64, Logical> = Point::from((150.0, 250.0));
        let current_location: Point<f64, Logical> = Point::from((160.0, 270.0));

        let delta = current_location - start_location;
        let new_location = initial.to_f64() + delta;

        assert_eq!(new_location.x, 110.0); // 100 + (160 - 150)
        assert_eq!(new_location.y, 220.0); // 200 + (270 - 250)
    }

    /// 测试初始位置不变时 delta 为零
    #[test]
    fn move_grab_zero_delta() {
        let initial: Point<i32, Logical> = Point::from((50, 75));
        let start: Point<f64, Logical> = Point::from((100.0, 100.0));
        let same: Point<f64, Logical> = Point::from((100.0, 100.0));

        let delta = same - start;
        let new_location = initial.to_f64() + delta;

        assert_eq!(new_location.x, 50.0);
        assert_eq!(new_location.y, 75.0);
    }

    /// 测试负方向位移
    #[test]
    fn move_grab_negative_delta() {
        let initial: Point<i32, Logical> = Point::from((300, 400));
        let start: Point<f64, Logical> = Point::from((200.0, 200.0));
        let current: Point<f64, Logical> = Point::from((180.0, 190.0));

        let delta = current - start;
        let new_location = initial.to_f64() + delta;

        assert_eq!(new_location.x, 280.0); // 300 + (180 - 200)
        assert_eq!(new_location.y, 390.0); // 400 + (190 - 200)
    }
}
