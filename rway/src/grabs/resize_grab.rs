// grabs/resize_grab.rs — 窗口调整大小抓取
//
// 用户拖拽窗口边框时，合成器进入 ResizeSurfaceGrab 状态。

use crate::state::RwayState;
use smithay::{
    desktop::{Space, Window},
    input::pointer::{
        AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent,
        GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
        GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData as PointerGrabStartData,
        MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent,
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Logical, Point, Rectangle, Size},
    wayland::{compositor, shell::xdg::SurfaceCachedState},
};
use std::cell::RefCell;

bitflags::bitflags! {
    /// 调整大小的边缘方向
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ResizeEdge: u32 {
        const TOP          = 0b0001;
        const BOTTOM       = 0b0010;
        const LEFT         = 0b0100;
        const RIGHT        = 0b1000;

        const TOP_LEFT     = Self::TOP.bits() | Self::LEFT.bits();
        const BOTTOM_LEFT  = Self::BOTTOM.bits() | Self::LEFT.bits();

        const TOP_RIGHT    = Self::TOP.bits() | Self::RIGHT.bits();
        const BOTTOM_RIGHT = Self::BOTTOM.bits() | Self::RIGHT.bits();
    }
}

impl From<xdg_toplevel::ResizeEdge> for ResizeEdge {
    #[inline]
    fn from(x: xdg_toplevel::ResizeEdge) -> Self {
        Self::from_bits(x as u32).unwrap_or(Self::empty())
    }
}

pub(crate) struct ResizeSurfaceGrab {
    start_data: PointerGrabStartData<RwayState>,
    window: Window,

    edges: ResizeEdge,

    initial_rect: Rectangle<i32, Logical>,
    last_window_size: Size<i32, Logical>,
}

impl ResizeSurfaceGrab {
    pub(crate) fn start(
        start_data: PointerGrabStartData<RwayState>,
        window: Window,
        edges: ResizeEdge,
        initial_window_rect: Rectangle<i32, Logical>,
    ) -> Option<Self> {
        let initial_rect = initial_window_rect;

        let toplevel = window.toplevel()?;
        ResizeSurfaceState::with(toplevel.wl_surface(), |state| {
            *state = ResizeSurfaceState::Resizing {
                edges,
                initial_rect,
            };
        });

        Some(Self {
            start_data,
            window,
            edges,
            initial_rect,
            last_window_size: initial_rect.size,
        })
    }
}

impl PointerGrab<RwayState> for ResizeSurfaceGrab {
    fn motion(
        &mut self,
        data: &mut RwayState,
        handle: &mut PointerInnerHandle<'_, RwayState>,
        _focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &MotionEvent,
    ) {
        // 抓取期间不给任何客户端指针焦点
        handle.motion(data, None, event);

        let mut delta = event.location - self.start_data.location;

        let mut new_window_width = self.initial_rect.size.w;
        let mut new_window_height = self.initial_rect.size.h;

        if self.edges.intersects(ResizeEdge::LEFT | ResizeEdge::RIGHT) {
            if self.edges.intersects(ResizeEdge::LEFT) {
                delta.x = -delta.x;
            }
            new_window_width = (self.initial_rect.size.w as f64 + delta.x) as i32;
        }

        if self.edges.intersects(ResizeEdge::TOP | ResizeEdge::BOTTOM) {
            if self.edges.intersects(ResizeEdge::TOP) {
                delta.y = -delta.y;
            }
            new_window_height = (self.initial_rect.size.h as f64 + delta.y) as i32;
        }

        let Some(toplevel) = self.window.toplevel() else {
            return;
        };
        let (min_size, max_size) = compositor::with_states(toplevel.wl_surface(), |states| {
            let mut guard = states.cached_state.get::<SurfaceCachedState>();
            let data = guard.current();
            (data.min_size, data.max_size)
        });

        let min_width = min_size.w.max(1);
        let min_height = min_size.h.max(1);
        let max_width = if max_size.w == 0 {
            i32::MAX
        } else {
            max_size.w
        };
        let max_height = if max_size.h == 0 {
            i32::MAX
        } else {
            max_size.h
        };

        self.last_window_size = Size::from((
            new_window_width.max(min_width).min(max_width),
            new_window_height.max(min_height).min(max_height),
        ));

        toplevel.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some(self.last_window_size);
        });
        toplevel.send_pending_configure();
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

        const BTN_LEFT: u32 = 0x110;

        if !handle.current_pressed().contains(&BTN_LEFT) {
            handle.unset_grab(self, data, event.serial, event.time, true);

            if let Some(xdg) = self.window.toplevel() {
                xdg.with_pending_state(|state| {
                    state.states.unset(xdg_toplevel::State::Resizing);
                    state.size = Some(self.last_window_size);
                });
                xdg.send_pending_configure();

                ResizeSurfaceState::with(xdg.wl_surface(), |state| {
                    *state = ResizeSurfaceState::WaitingForLastCommit {
                        edges: self.edges,
                        initial_rect: self.initial_rect,
                    };
                });
            }
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

/// 调整大小操作的状态，存储在 WlSurface 的 data_map 中
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
enum ResizeSurfaceState {
    #[default]
    Idle,
    Resizing {
        edges: ResizeEdge,
        /// 初始窗口矩形（位置 + 大小）
        initial_rect: Rectangle<i32, Logical>,
    },
    /// 调整完成，等待最后一次 commit 以执行最终移动
    WaitingForLastCommit {
        edges: ResizeEdge,
        initial_rect: Rectangle<i32, Logical>,
    },
}

impl ResizeSurfaceState {
    fn with<F, T>(surface: &WlSurface, cb: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        compositor::with_states(surface, |states| {
            states.data_map.insert_if_missing(RefCell::<Self>::default);
            let state = states
                .data_map
                .get::<RefCell<Self>>()
                .expect("just inserted");
            cb(&mut state.borrow_mut())
        })
    }

    fn commit(&mut self) -> Option<(ResizeEdge, Rectangle<i32, Logical>)> {
        match *self {
            Self::Resizing {
                edges,
                initial_rect,
            } => Some((edges, initial_rect)),
            Self::WaitingForLastCommit {
                edges,
                initial_rect,
            } => {
                *self = Self::Idle;
                Some((edges, initial_rect))
            }
            Self::Idle => None,
        }
    }
}

/// Called on `WlSurface::commit`: handle window position correction after resize
pub(crate) fn handle_commit(space: &mut Space<Window>, surface: &WlSurface) -> Option<()> {
    let window = space
        .elements()
        .find(|w| {
            w.toplevel()
                .map(|t| t.wl_surface() == surface)
                .unwrap_or(false)
        })
        .cloned()?;

    let mut window_loc = space.element_location(&window)?;
    let geometry = window.geometry();

    let new_loc: Point<Option<i32>, Logical> = ResizeSurfaceState::with(surface, |state| {
        state
            .commit()
            .and_then(|(edges, initial_rect)| {
                // 若从顶部或左侧调整，需相应移动窗口位置
                edges.intersects(ResizeEdge::TOP_LEFT).then(|| {
                    let new_x = edges
                        .intersects(ResizeEdge::LEFT)
                        .then_some(initial_rect.loc.x + (initial_rect.size.w - geometry.size.w));
                    let new_y = edges
                        .intersects(ResizeEdge::TOP)
                        .then_some(initial_rect.loc.y + (initial_rect.size.h - geometry.size.h));
                    (new_x, new_y).into()
                })
            })
            .unwrap_or_default()
    });

    if let Some(new_x) = new_loc.x {
        window_loc.x = new_x;
    }
    if let Some(new_y) = new_loc.y {
        window_loc.y = new_y;
    }

    if new_loc.x.is_some() || new_loc.y.is_some() {
        space.map_element(window, window_loc, false);
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use super::ResizeEdge;
    use smithay::utils::{Logical, Rectangle, Size};

    // ── ResizeEdge 位标志测试 ──────────────────────────────────

    #[test]
    fn resize_edge_top_flag() {
        assert_eq!(ResizeEdge::TOP.bits(), 0b0001);
    }

    #[test]
    fn resize_edge_bottom_flag() {
        assert_eq!(ResizeEdge::BOTTOM.bits(), 0b0010);
    }

    #[test]
    fn resize_edge_left_flag() {
        assert_eq!(ResizeEdge::LEFT.bits(), 0b0100);
    }

    #[test]
    fn resize_edge_right_flag() {
        assert_eq!(ResizeEdge::RIGHT.bits(), 0b1000);
    }

    #[test]
    fn resize_edge_top_left_is_combination() {
        let tl = ResizeEdge::TOP_LEFT;
        assert!(tl.contains(ResizeEdge::TOP));
        assert!(tl.contains(ResizeEdge::LEFT));
        assert!(!tl.contains(ResizeEdge::BOTTOM));
        assert!(!tl.contains(ResizeEdge::RIGHT));
    }

    #[test]
    fn resize_edge_bottom_right_is_combination() {
        let br = ResizeEdge::BOTTOM_RIGHT;
        assert!(br.contains(ResizeEdge::BOTTOM));
        assert!(br.contains(ResizeEdge::RIGHT));
        assert!(!br.contains(ResizeEdge::TOP));
        assert!(!br.contains(ResizeEdge::LEFT));
    }

    #[test]
    fn resize_edge_intersects_left_or_right() {
        let left = ResizeEdge::LEFT;
        assert!(left.intersects(ResizeEdge::LEFT | ResizeEdge::RIGHT));

        let right = ResizeEdge::RIGHT;
        assert!(right.intersects(ResizeEdge::LEFT | ResizeEdge::RIGHT));

        let top = ResizeEdge::TOP;
        assert!(!top.intersects(ResizeEdge::LEFT | ResizeEdge::RIGHT));
    }

    #[test]
    fn resize_edge_empty_does_not_intersect() {
        let empty = ResizeEdge::empty();
        assert!(!empty.intersects(ResizeEdge::TOP | ResizeEdge::BOTTOM));
        assert!(!empty.intersects(ResizeEdge::LEFT | ResizeEdge::RIGHT));
    }

    // ── 尺寸约束计算测试 ───────────────────────────────────────

    /// 测试右侧/底部拖拽的新尺寸计算
    #[test]
    fn resize_new_size_right_bottom() {
        let initial_w = 800_i32;
        let initial_h = 600_i32;
        let delta_x = 50.0_f64;
        let delta_y = 30.0_f64;

        // 从右下角拖拽：宽高增加
        let new_w = (initial_w as f64 + delta_x) as i32;
        let new_h = (initial_h as f64 + delta_y) as i32;

        assert_eq!(new_w, 850);
        assert_eq!(new_h, 630);
    }

    /// 测试左侧拖拽时 delta 取反（从左侧拖拽宽度增加 = 鼠标向左移动）
    #[test]
    fn resize_left_edge_inverts_delta_x() {
        let initial_w = 800_i32;
        let mut delta_x = -50.0_f64; // 鼠标向左移动

        // 左边缘：取反
        delta_x = -delta_x; // = 50.0

        let new_w = (initial_w as f64 + delta_x) as i32;
        assert_eq!(new_w, 850); // 宽度增加
    }

    /// 测试尺寸下限约束（最小 1 像素）
    #[test]
    fn resize_clamp_to_min_size() {
        let new_w = -100_i32;
        let min_width = 1_i32.max(0); // min_size.w=0 时使用 1
        let clamped = new_w.max(min_width);
        assert_eq!(clamped, 1);
    }

    /// 测试尺寸上限约束
    #[test]
    fn resize_clamp_to_max_size() {
        let new_w = 5000_i32;
        let max_width = 1920_i32; // max_size.w != 0
        let clamped = new_w.min(max_width);
        assert_eq!(clamped, 1920);
    }

    /// 测试 max_size=0 时视为无上限
    #[test]
    fn resize_max_size_zero_means_unlimited() {
        let max_size_w = 0_i32;
        let effective_max = if max_size_w == 0 {
            i32::MAX
        } else {
            max_size_w
        };
        assert_eq!(effective_max, i32::MAX);
    }

    // ── 位置修正测试（左/顶边缘调整时需移动窗口） ─────────────

    /// 测试从左边缘调整大小时的 x 坐标修正
    #[test]
    fn resize_position_correction_left_edge() {
        let initial_rect: Rectangle<i32, Logical> =
            Rectangle::new((100, 200).into(), (800, 600).into());
        let new_geometry_size: Size<i32, Logical> = Size::from((850, 600));

        // x 新坐标 = 初始 x + (初始宽 - 新宽)
        let new_x = initial_rect.loc.x + (initial_rect.size.w - new_geometry_size.w);
        assert_eq!(new_x, 50); // 100 + (800 - 850) = 50
    }

    /// 测试从顶边缘调整大小时的 y 坐标修正
    #[test]
    fn resize_position_correction_top_edge() {
        let initial_rect: Rectangle<i32, Logical> =
            Rectangle::new((100, 200).into(), (800, 600).into());
        let new_geometry_size: Size<i32, Logical> = Size::from((800, 650));

        // y 新坐标 = 初始 y + (初始高 - 新高)
        let new_y = initial_rect.loc.y + (initial_rect.size.h - new_geometry_size.h);
        assert_eq!(new_y, 150); // 200 + (600 - 650) = 150
    }

    /// 测试右/底边缘调整时不需要修正位置
    #[test]
    fn resize_no_position_correction_right_bottom() {
        let edges = ResizeEdge::BOTTOM_RIGHT;
        // BOTTOM_RIGHT 不包含 TOP_LEFT，所以不需要位置修正
        assert!(!edges.intersects(ResizeEdge::TOP_LEFT));
    }

    // ── ResizeSurfaceState 状态机测试 ─────────────────────────

    /// 测试 Resizing 状态的 commit 返回边缘和矩形
    #[test]
    fn resize_state_resizing_commit_returns_data() {
        use super::*;

        let edges = ResizeEdge::RIGHT;
        let rect: Rectangle<i32, Logical> = Rectangle::new((0, 0).into(), (400, 300).into());
        let mut state = ResizeSurfaceState::Resizing {
            edges,
            initial_rect: rect,
        };

        // Resizing 状态下 commit 返回数据且状态不变
        let result = state.commit();
        assert!(result.is_some());
        let (e, r) = result.unwrap();
        assert_eq!(e, edges);
        assert_eq!(r, rect);
        // 状态仍为 Resizing
        assert!(matches!(state, ResizeSurfaceState::Resizing { .. }));
    }

    /// 测试 WaitingForLastCommit 状态的 commit 转换为 Idle
    #[test]
    fn resize_state_waiting_commit_transitions_to_idle() {
        use super::*;

        let edges = ResizeEdge::BOTTOM;
        let rect: Rectangle<i32, Logical> = Rectangle::new((0, 0).into(), (200, 150).into());
        let mut state = ResizeSurfaceState::WaitingForLastCommit {
            edges,
            initial_rect: rect,
        };

        let result = state.commit();
        assert!(result.is_some());
        // 转换为 Idle
        assert_eq!(state, ResizeSurfaceState::Idle);
    }

    /// 测试 Idle 状态的 commit 返回 None
    #[test]
    fn resize_state_idle_commit_returns_none() {
        use super::*;

        let mut state = ResizeSurfaceState::Idle;
        let result = state.commit();
        assert!(result.is_none());
    }
}
