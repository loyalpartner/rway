// RwayState — rway 合成器的核心状态
// 持有所有 Smithay 协议状态、平铺引擎、配置、IPC

use std::{collections::HashMap, ffi::OsString, sync::Arc};

use smithay::{
    desktop::{layer_map_for_output, PopupManager, Space, Window, WindowSurfaceType},
    input::{pointer::CursorImageStatus, Seat, SeatState},
    reexports::{
        calloop::{
            generic::Generic, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction,
        },
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        input_method::InputMethodManagerState,
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{decoration::XdgDecorationState, XdgShellState},
        },
        shm::ShmState,
        socket::ListeningSocketSource,
        text_input::TextInputManagerState,
        virtual_keyboard::VirtualKeyboardManagerState,
    },
};

#[cfg(feature = "xwayland")]
use smithay::wayland::xwayland_shell::XWaylandShellState;
#[cfg(feature = "xwayland")]
use smithay::xwayland::X11Wm;

use smithay::utils::IsAlive;

use rway_tiling::{layout, workspace, NodeId, Rect, Tree};

use crate::animation::{AnimationConfig, AnimationManager};

/// rway 合成器的主状态结构体
pub struct RwayState {
    pub(crate) start_time: std::time::Instant,
    pub(crate) socket_name: OsString,
    pub(crate) display_handle: DisplayHandle,

    pub(crate) space: Space<Window>,
    pub(crate) loop_signal: LoopSignal,

    // Smithay protocol state
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell_state: XdgShellState,
    pub(crate) shm_state: ShmState,
    // Kept alive for protocol side effects; not read directly.
    #[allow(dead_code)]
    pub(crate) output_manager_state: OutputManagerState,
    pub(crate) seat_state: SeatState<RwayState>,
    pub(crate) data_device_state: DataDeviceState,
    pub(crate) layer_shell_state: WlrLayerShellState,
    // Kept alive for protocol side effects; not read directly.
    #[allow(dead_code)]
    pub(crate) xdg_decoration_state: XdgDecorationState,
    pub(crate) popups: PopupManager,
    pub(crate) seat: Seat<Self>,

    // Input method (IME) protocol state
    // Kept alive for protocol side effects; not read directly.
    #[allow(dead_code)]
    pub(crate) text_input_manager_state: TextInputManagerState,
    #[allow(dead_code)]
    pub(crate) input_method_manager_state: InputMethodManagerState,
    #[allow(dead_code)]
    pub(crate) virtual_keyboard_manager_state: VirtualKeyboardManagerState,

    // Tiling engine
    pub(crate) tiling: Tree,
    pub(crate) window_map: HashMap<u64, Window>,
    pub(crate) next_window_id: u64,
    pub(crate) output_node: Option<NodeId>,

    // Animation manager
    pub(crate) animations: AnimationManager,

    // Cached set of visible window IDs (updated in relayout, consumed in update_animations)
    pub(crate) visible_windows: std::collections::HashSet<u64>,

    // Cached title bar rects (updated in relayout, consumed in render)
    pub(crate) cached_title_bars: Vec<rway_tiling::TitleBar>,

    // Title bar text renderer (cosmic-text FontSystem + cache)
    pub(crate) text_renderer: crate::text::TextRenderer,

    // Cursor state (updated by client via SeatHandler::cursor_image callback)
    pub(crate) cursor_status: CursorImageStatus,

    // Configuration
    pub(crate) config: rway_config::Config,

    // Event loop handle (for scheduling idle callbacks and timers)
    pub(crate) loop_handle: LoopHandle<'static, RwayState>,

    // Redraw scheduling — set to true when content changes, cleared after rendering
    pub(crate) needs_redraw: bool,

    // Winit backend state (None when using udev backend)
    pub(crate) winit: Option<crate::backend::winit::WinitState>,
    // Ping to trigger a render frame (winit backend only)
    pub(crate) render_ping: Option<smithay::reexports::calloop::ping::Ping>,

    // IPC
    pub(crate) ipc_server: Option<rway_ipc::IpcServer>,
    pub(crate) ipc_clients: Vec<crate::ipc::IpcClient>,

    // XWayland support (only used when xwayland feature is enabled)
    #[cfg(feature = "xwayland")]
    pub(crate) xwayland_shell_state: Option<XWaylandShellState>,
    #[cfg(feature = "xwayland")]
    pub(crate) xwm: Option<X11Wm>,
    #[cfg(feature = "xwayland")]
    pub(crate) xdisplay: Option<u32>,

    // Udev backend data (only used when udev feature is enabled)
    #[cfg(feature = "udev")]
    pub(crate) udev_data: Option<crate::backend::udev::UdevData>,
}

impl RwayState {
    /// 创建新的 RwayState，初始化所有 Smithay 协议状态（使用默认 seat 名称 "winit"）
    pub fn new(event_loop: &mut EventLoop<'static, Self>, display: Display<Self>) -> Self {
        Self::new_with_seat_name(event_loop, display, "winit")
    }

    /// 创建新的 RwayState，使用指定的 seat 名称
    pub fn new_with_seat_name(
        event_loop: &mut EventLoop<'static, Self>,
        display: Display<Self>,
        seat_name: &str,
    ) -> Self {
        let start_time = std::time::Instant::now();

        let dh = display.handle();

        // 初始化合成器协议
        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let popups = PopupManager::default();

        // 支持 xdg-output 扩展
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);

        // 剪贴板和拖拽协议
        let data_device_state = DataDeviceState::new::<Self>(&dh);

        // Layer Shell 协议（waybar、swaybg 等使用）
        let layer_shell_state = WlrLayerShellState::new::<Self>(&dh);

        // XDG Decoration 协议（告知客户端由合成器处理装饰）
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&dh);

        // Input method (IME) protocols: text-input, input-method, virtual-keyboard
        let text_input_manager_state = TextInputManagerState::new::<Self>(&dh);
        let input_method_manager_state =
            InputMethodManagerState::new::<Self, _>(&dh, |_client| true);
        let virtual_keyboard_manager_state =
            VirtualKeyboardManagerState::new::<Self, _>(&dh, |_client| true);

        // Seat：键盘、指针、触摸设备的逻辑组合
        let mut seat_state = SeatState::new();
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, seat_name);

        // 键盘：默认使用 Dvorak 布局
        let xkb_config = smithay::input::keyboard::XkbConfig {
            layout: "us",
            variant: "dvorak",
            ..Default::default()
        };
        if let Err(e) = seat.add_keyboard(xkb_config, 200, 25) {
            tracing::error!("Failed to add keyboard to seat: {}", e);
        }
        seat.add_pointer();

        // 二维平面空间，窗口和输出都映射在上面
        let space = Space::default();

        // 建立 Wayland 套接字监听
        let socket_name = Self::init_wayland_listener(display, event_loop);

        // 事件循环停止信号
        let loop_signal = event_loop.get_signal();

        // 事件循环句柄
        let loop_handle = event_loop.handle();

        // 初始化平铺引擎
        let tiling = Tree::new();

        // 初始化动画管理器
        let animations = AnimationManager::new(AnimationConfig::default());

        // 加载配置
        let config = Self::load_config();

        // 启动 IPC 服务器
        let ipc_server = Self::init_ipc_server();

        Self {
            start_time,
            display_handle: dh,

            space,
            loop_signal,
            socket_name,

            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            layer_shell_state,
            xdg_decoration_state,
            popups,
            seat,

            text_input_manager_state,
            input_method_manager_state,
            virtual_keyboard_manager_state,

            tiling,
            window_map: HashMap::new(),
            next_window_id: 1,
            output_node: None,

            animations,
            visible_windows: std::collections::HashSet::new(),
            cached_title_bars: Vec::new(),
            text_renderer: crate::text::TextRenderer::new(crate::text::FontConfig::from_sway_font(
                config.font.as_deref(),
            )),

            cursor_status: CursorImageStatus::default_named(),

            loop_handle,

            config,
            needs_redraw: true,
            winit: None,
            render_ping: None,
            ipc_server,
            ipc_clients: Vec::new(),

            #[cfg(feature = "xwayland")]
            xwayland_shell_state: None,
            #[cfg(feature = "xwayland")]
            xwm: None,
            #[cfg(feature = "xwayland")]
            xdisplay: None,

            #[cfg(feature = "udev")]
            udev_data: None,
        }
    }

    /// 在输出创建后调用，在平铺树中注册输出和默认工作区
    pub fn init_tiling_output(&mut self, width: i32, height: i32) {
        self.init_tiling_output_named("default", width, height);
    }

    /// 在输出创建后调用，使用指定名称在平铺树中注册输出和默认工作区
    pub fn init_tiling_output_named(&mut self, name: &str, width: i32, height: i32) {
        let output_id =
            workspace::add_output(&mut self.tiling, name, Rect::new(0, 0, width, height));
        self.output_node = Some(output_id);

        // 创建默认工作区 "1"
        workspace::add_workspace(&mut self.tiling, output_id, "1");
    }

    /// Return current border width from config (0 for BorderStyle::None).
    pub(crate) fn border_width(&self) -> i32 {
        match &self.config.default_border {
            rway_config::BorderStyle::Normal(w) | rway_config::BorderStyle::Pixel(w) => *w as i32,
            rway_config::BorderStyle::None => 0,
        }
    }

    /// Mark content as changed. The winit ping cycle picks this up
    /// on the next frame. For udev, VBlank-driven.
    pub fn schedule_redraw(&mut self) {
        self.needs_redraw = true;
        if let Some(ping) = &self.render_ping {
            ping.ping();
        }
    }

    /// Default title bar height when not configured (pixels).
    const DEFAULT_TITLE_BAR_HEIGHT: i32 = 25;

    /// Build tiling GapsConfig from current config values.
    pub(crate) fn gaps_config(&self) -> rway_tiling::GapsConfig {
        rway_tiling::GapsConfig {
            inner: self.config.gaps.inner as i32,
            outer: self.config.gaps.outer as i32,
            title_bar_height: Self::DEFAULT_TITLE_BAR_HEIGHT,
        }
    }

    /// 重新计算平铺布局并更新 Space 中窗口的位置和大小
    pub fn relayout(&mut self) {
        // 获取输出的非 exclusive 区域（扣除 waybar 等 layer shell 客户端占用的空间）
        let output_geo = if let Some(output) = self.space.outputs().next().cloned() {
            let layer_map = layer_map_for_output(&output);
            layer_map.non_exclusive_zone()
        } else {
            smithay::utils::Rectangle::new((0, 0).into(), (1920, 1080).into())
        };

        let available = Rect::new(
            output_geo.loc.x,
            output_geo.loc.y,
            output_geo.size.w,
            output_geo.size.h,
        );

        // 计算布局（从配置读取 gaps）
        let root = self.tiling.root();
        let gaps = self.gaps_config();
        layout::compute_layout(&mut self.tiling, root, available, &gaps);

        // Border width: window content is inset by this amount on each side
        let bw = self.border_width();

        // 获取所有窗口的几何并设置动画目标
        let geometries = layout::get_window_geometries(&self.tiling);
        let mut raised_windows: Vec<Window> = Vec::new();

        // Rebuild visibility cache once per relayout (O(N×depth)),
        // so update_animations() can check O(1) per window per frame.
        self.visible_windows.clear();
        for &(wid, _) in &geometries {
            if self.tiling.is_visible(wid) {
                self.visible_windows.insert(wid);
            }
        }

        for (window_id, rect) in geometries {
            let is_fs = self.tiling.is_fullscreen(window_id);
            let is_float = self.tiling.is_floating(window_id);
            let is_special = is_fs || is_float;
            let visible = self.visible_windows.contains(&window_id);

            tracing::debug!(
                window_id,
                visible,
                x = rect.x,
                y = rect.y,
                w = rect.width,
                h = rect.height,
                "relayout window geometry"
            );

            self.animations
                .set_target(window_id, rect.x, rect.y, rect.width, rect.height);

            if let Some(window) = self.window_map.get(&window_id) {
                // Hide non-visible windows (non-focused children in Tabbed/Stacked)
                if !visible {
                    self.space.unmap_elem(window);
                    continue;
                }

                // Fullscreen/floating: no tiling border offset
                // Normal tiled: inset by border_width on each side
                let map_bw = if is_special { 0 } else { bw };
                let content_w = (rect.width - 2 * map_bw).max(1);
                let content_h = (rect.height - 2 * map_bw).max(1);

                if let Some(toplevel) = window.toplevel() {
                    use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
                    toplevel.with_pending_state(|state| {
                        state.size = Some((content_w, content_h).into());
                        if is_fs {
                            state.states.set(xdg_toplevel::State::Fullscreen);
                        } else {
                            state.states.unset(xdg_toplevel::State::Fullscreen);
                        }
                    });
                    toplevel.send_pending_configure();
                }

                // Use tiling engine geometry directly for positioning.
                // Animation positions lag behind (set_target doesn't update
                // current_positions until tick), causing Tabbed/Stacked windows
                // to appear at their old SplitH/V positions.
                self.space
                    .map_element(window.clone(), (rect.x + map_bw, rect.y + map_bw), false);

                // Floating and fullscreen windows are raised above tiled
                if is_special {
                    raised_windows.push(window.clone());
                }
            }
        }

        // Raise floating/fullscreen AFTER all tiled windows are mapped.
        // Fullscreen on top of floating on top of tiled.
        for w in &raised_windows {
            self.space.raise_element(w, true);
        }

        // Cache title bar rects so render doesn't re-traverse tree every frame
        self.cached_title_bars = self.tiling.title_bars(&gaps);

        // Flush configure events to clients immediately so they can
        // start redrawing (e.g. fullscreen size change).
        let _ = self.display_handle.flush_clients();

        // Layout changed — schedule redraw
        self.schedule_redraw();
    }

    /// 确保当前聚焦的工作区在一个活跃（有映射到 Space 中的）输出上。
    /// 如果不是，将焦点切换到第一个活跃输出的工作区。
    pub fn ensure_active_workspace(&mut self) {
        // 获取 Space 中所有活跃输出的名称
        let active_outputs: Vec<String> = self.space.outputs().map(|o| o.name()).collect();

        if active_outputs.is_empty() {
            return;
        }

        // 检查当前聚焦工作区是否在活跃输出上
        let focused_ws = rway_tiling::workspace::get_focused_workspace(&self.tiling);
        if let Some(ws_id) = focused_ws {
            if let Some(node) = self.tiling.get(ws_id) {
                if let rway_tiling::NodeData::Workspace { output, .. } = &node.data {
                    if let Some(out_node) = self.tiling.get(*output) {
                        if let rway_tiling::NodeData::Output { name, .. } = &out_node.data {
                            if active_outputs.contains(name) {
                                return; // 已经在活跃输出上
                            }
                        }
                    }
                }
            }
        }

        // 聚焦工作区不在活跃输出上，找到一个活跃输出的工作区并切换
        let workspaces = rway_tiling::workspace::get_workspaces(&self.tiling);
        for (ws_id, ws_name, _) in &workspaces {
            if let Some(node) = self.tiling.get(*ws_id) {
                if let rway_tiling::NodeData::Workspace { output, .. } = &node.data {
                    if let Some(out_node) = self.tiling.get(*output) {
                        if let rway_tiling::NodeData::Output { name, .. } = &out_node.data {
                            if active_outputs.contains(name) {
                                rway_tiling::workspace::switch_workspace(&mut self.tiling, ws_name);
                                tracing::info!("已将焦点切换到活跃输出上的工作区: {}", ws_name);
                                return;
                            }
                        }
                    }
                }
            }
        }

        // 没有活跃输出上的工作区，在第一个活跃输出上创建默认工作区
        let first_output = &active_outputs[0];
        let root = self.tiling.root();
        for &output_id in self.tiling.children(root).to_vec().iter() {
            if let Some(node) = self.tiling.get(output_id) {
                if let rway_tiling::NodeData::Output { name, .. } = &node.data {
                    if name == first_output {
                        rway_tiling::workspace::add_workspace(&mut self.tiling, output_id, "1");
                        rway_tiling::workspace::switch_workspace(&mut self.tiling, "1");
                        tracing::info!("在活跃输出 {} 上创建了默认工作区", first_output);
                        return;
                    }
                }
            }
        }
    }

    /// 每帧调用：推进动画插值并更新 Space 中窗口的渲染位置
    ///
    /// 应在渲染之前调用。动画管理器会根据时间推进所有活跃动画的插值，
    /// 然后将计算出的中间帧位置应用到 Space。
    /// Returns true if animations are still active (caller should request redraw)
    pub fn update_animations(&mut self) -> bool {
        let has_active = self.animations.tick();
        let bw = self.border_width();

        // Apply interpolated positions to Space.
        // Floating/fullscreen: no border offset, raise to top.
        // Normal tiled: inset by border_width.
        let mut raised: Vec<Window> = Vec::new();
        for (&window_id, window) in &self.window_map {
            // Skip non-visible windows (hidden tabs in Tabbed/Stacked).
            // Uses cached set from relayout() — O(1) instead of per-frame tree walk.
            if !self.visible_windows.contains(&window_id) {
                continue;
            }
            if let Some((x, y, _w, _h)) = self.animations.get_position(window_id) {
                let is_special =
                    self.tiling.is_fullscreen(window_id) || self.tiling.is_floating(window_id);
                let offset = if is_special { 0 } else { bw };
                self.space
                    .map_element(window.clone(), (x + offset, y + offset), false);
                if is_special {
                    raised.push(window.clone());
                }
            }
        }
        for w in &raised {
            self.space.raise_element(w, true);
        }

        has_active
    }

    /// 检测并清理已关闭的窗口：从平铺树和 window_map 中移除死亡窗口
    ///
    /// 应在每帧 `space.refresh()` 之后调用。
    pub fn cleanup_dead_windows(&mut self) {
        // 收集已死亡的窗口 ID（使用 IsAlive trait 统一判断 Wayland 和 X11 窗口）
        let dead_ids: Vec<u64> = self
            .window_map
            .iter()
            .filter(|(_, window)| !window.alive())
            .map(|(id, _)| *id)
            .collect();

        if dead_ids.is_empty() {
            return;
        }

        // 从平铺树、window_map 和动画管理器中移除
        for id in &dead_ids {
            rway_tiling::commands::remove_window(&mut self.tiling, *id);
            self.window_map.remove(id);
            self.animations.remove(*id);
        }

        tracing::debug!("清理了 {} 个已关闭窗口", dead_ids.len());

        // 重新布局以填充空出的空间
        self.relayout();

        // 将焦点转移到下一个窗口
        crate::focus::update_focus(self);
    }

    /// 分配下一个窗口 ID
    pub fn alloc_window_id(&mut self) -> u64 {
        let id = self.next_window_id;
        self.next_window_id += 1;
        id
    }

    /// 内置默认配置（编译时嵌入 config/default 文件）
    const DEFAULT_CONFIG: &'static str = include_str!("../../config/default");

    /// 加载配置文件
    ///
    /// 优先级: ~/.config/rway/config → ~/.config/sway/config → 内置默认配置
    pub fn load_config() -> rway_config::Config {
        let config_paths = [
            dirs_config_path("rway/config"),
            dirs_config_path("sway/config"),
        ];

        for path in config_paths.into_iter().flatten() {
            if path.exists() {
                match rway_config::parse_file(&path) {
                    Ok(config) => {
                        tracing::info!("已加载配置: {:?}", path);
                        tracing::info!("已加载 {} 个快捷键", config.keybindings.len());
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("配置解析失败 {:?}: {}", path, e);
                    }
                }
            }
        }

        // 所有外部配置文件都不存在或解析失败，使用内置默认配置
        tracing::info!("使用内置默认配置");
        match rway_config::parse(Self::DEFAULT_CONFIG) {
            Ok(config) => {
                tracing::info!("已加载 {} 个快捷键（内置默认）", config.keybindings.len());
                config
            }
            Err(e) => {
                tracing::error!("内置默认配置解析失败（这不应该发生）: {}", e);
                rway_config::Config::default()
            }
        }
    }

    /// 启动 IPC 服务器
    fn init_ipc_server() -> Option<rway_ipc::IpcServer> {
        let socket_path = rway_ipc::default_socket_path();
        match rway_ipc::IpcServer::new(socket_path) {
            Ok(server) => {
                tracing::info!("IPC 服务器已启动: {:?}", server.socket_path());
                // 设置 $SWAYSOCK 环境变量以兼容 sway 工具链
                std::env::set_var("SWAYSOCK", server.socket_path());
                Some(server)
            }
            Err(e) => {
                tracing::warn!("IPC 服务器启动失败: {}", e);
                None
            }
        }
    }

    /// 启动 XWayland 服务器，提供 X11 应用兼容性
    #[cfg(feature = "xwayland")]
    pub fn start_xwayland(&mut self) {
        use std::process::Stdio;

        use smithay::wayland::compositor::CompositorHandler;
        use smithay::xwayland::{XWayland, XWaylandEvent};

        // 初始化 XWayland Shell 协议状态
        self.xwayland_shell_state = Some(XWaylandShellState::new::<Self>(
            &self.display_handle.clone(),
        ));

        // 提前探测空闲的 X11 display 编号并设置 DISPLAY 环境变量，
        // 这样在 XWayland Ready 之前启动的子进程也能继承正确的 DISPLAY
        let display_num = find_free_x11_display();
        std::env::set_var("DISPLAY", format!(":{}", display_num));
        tracing::info!("预设 DISPLAY=:{}", display_num);

        // 启动 XWayland 进程，指定探测到的 display 编号
        let (xwayland, client) = match XWayland::spawn(
            &self.display_handle,
            Some(display_num),
            std::iter::empty::<(String, String)>(),
            true,
            Stdio::null(),
            Stdio::null(),
            |_| (),
        ) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("启动 XWayland 失败: {}（X11 应用将不可用）", e);
                return;
            }
        };

        // 将 XWayland 事件源注册到事件循环
        let ret = self
            .loop_handle
            .insert_source(xwayland, move |event, _, data| match event {
                XWaylandEvent::Ready {
                    x11_socket,
                    display_number,
                } => {
                    // 设置 XWayland 客户端缩放比例
                    let xwayland_scale = std::env::var("RWAY_XWAYLAND_SCALE")
                        .ok()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    data.client_compositor_state(&client)
                        .set_client_scale(xwayland_scale);

                    // 启动 X11 窗口管理器
                    let mut wm =
                        match X11Wm::start_wm(data.loop_handle.clone(), x11_socket, client.clone())
                        {
                            Ok(wm) => wm,
                            Err(e) => {
                                tracing::error!("启动 X11 窗口管理器失败: {}", e);
                                return;
                            }
                        };

                    // 设置 XWayland 默认光标（使用 xcursor 主题）
                    if let Err(e) = Self::set_xwayland_cursor(&mut wm) {
                        tracing::warn!("设置 XWayland 光标失败: {}（将使用默认 X 光标）", e);
                    }

                    data.xwm = Some(wm);
                    data.xdisplay = Some(display_number);
                    tracing::info!("XWayland 已就绪，DISPLAY=:{}", display_number);
                }
                XWaylandEvent::Error => {
                    tracing::warn!("XWayland 启动时崩溃");
                }
            });

        if let Err(e) = ret {
            tracing::error!("将 XWayland 事件源注册到事件循环失败: {}", e);
        }
    }

    /// 设置 XWayland 的默认光标图像
    #[cfg(feature = "xwayland")]
    fn set_xwayland_cursor(wm: &mut X11Wm) -> Result<(), Box<dyn std::error::Error>> {
        use smithay::utils::{Point, Size};

        let cursor_size = std::env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(24);
        let cursor_theme = std::env::var("XCURSOR_THEME")
            .ok()
            .unwrap_or_else(|| "default".into());

        let theme = xcursor::CursorTheme::load(&cursor_theme);
        let icon_path = theme
            .load_icon("default")
            .ok_or("光标主题中找不到 default 图标")?;
        let mut cursor_data = Vec::new();
        std::io::Read::read_to_end(&mut std::fs::File::open(icon_path)?, &mut cursor_data)?;
        let images = xcursor::parser::parse_xcursor(&cursor_data).ok_or("解析 xcursor 文件失败")?;

        // 选择最接近请求大小的图标
        let image = images
            .iter()
            .min_by_key(|img| (cursor_size as i32 - img.size as i32).abs())
            .ok_or("xcursor 文件中没有图像")?;

        wm.set_cursor(
            &image.pixels_rgba,
            Size::from((image.width as u16, image.height as u16)),
            Point::from((image.xhot as u16, image.yhot as u16)),
        )?;

        Ok(())
    }

    /// 初始化 Wayland 套接字监听源，注册到事件循环
    fn init_wayland_listener(
        display: Display<RwayState>,
        event_loop: &mut EventLoop<Self>,
    ) -> OsString {
        let listening_socket =
            ListeningSocketSource::new_auto().expect("Failed to create Wayland listening socket");
        let socket_name = listening_socket.socket_name().to_os_string();

        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                if let Err(e) = state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                {
                    tracing::warn!("Failed to insert Wayland client: {}", e);
                }
            })
            .expect("初始化 Wayland 事件源失败");

        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    unsafe {
                        if let Err(e) = display.get_mut().dispatch_clients(state) {
                            tracing::warn!("Failed to dispatch Wayland clients: {}", e);
                        }
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("Failed to register Wayland display event source");

        socket_name
    }

    /// 在给定逻辑坐标 `pos` 下，找到对应的 WlSurface 及其相对坐标
    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, (p + location).to_f64()))
            })
    }
}

/// 获取 XDG 配置路径
fn dirs_config_path(relative: &str) -> Option<std::path::PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".config"))
        })
        .map(|base| base.join(relative))
}

/// 探测空闲的 X11 display 编号（检查 /tmp/.X{N}-lock 和 /tmp/.X11-unix/X{N}）
#[cfg(feature = "xwayland")]
fn find_free_x11_display() -> u32 {
    for n in 0..32 {
        let lock = format!("/tmp/.X{}-lock", n);
        let socket = format!("/tmp/.X11-unix/X{}", n);
        if !std::path::Path::new(&lock).exists() && !std::path::Path::new(&socket).exists() {
            return n;
        }
    }
    // 回退到一个大编号
    99
}

/// 每个连接到 rway 的 Wayland 客户端关联的数据
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

#[cfg(test)]
mod tests {
    use super::ClientState;
    use smithay::wayland::compositor::CompositorClientState;

    #[test]
    fn client_state_default_is_valid() {
        let cs = ClientState::default();
        let _: &CompositorClientState = &cs.compositor_state;
    }
}
