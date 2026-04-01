// RwayState — rway 合成器的核心状态
// 持有所有 Smithay 协议状态、平铺引擎、配置、IPC

use std::{collections::HashMap, ffi::OsString, sync::Arc};

use smithay::{
    desktop::{layer_map_for_output, PopupManager, Space, Window, WindowSurfaceType},
    input::{Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Logical, Point},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{
                decoration::XdgDecorationState,
                XdgShellState,
            },
        },
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

use rway_tiling::{layout, workspace, NodeId, Rect, Tree};

/// rway 合成器的主状态结构体
pub struct RwayState {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,
    pub display_handle: DisplayHandle,

    pub space: Space<Window>,
    pub loop_signal: LoopSignal,

    // Smithay 协议状态
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<RwayState>,
    pub data_device_state: DataDeviceState,
    pub layer_shell_state: WlrLayerShellState,
    pub xdg_decoration_state: XdgDecorationState,
    pub popups: PopupManager,
    pub seat: Seat<Self>,

    // 平铺引擎
    pub tiling: Tree,
    pub window_map: HashMap<u64, Window>,
    pub next_window_id: u64,
    pub output_node: Option<NodeId>,

    // 配置
    pub config: rway_config::Config,

    // 事件循环句柄（用于调度空闲回调和定时器）
    pub loop_handle: LoopHandle<'static, RwayState>,

    // IPC
    pub ipc_server: Option<rway_ipc::IpcServer>,

    // Udev 后端数据（仅在 udev feature 启用时使用）
    #[cfg(feature = "udev")]
    pub udev_data: Option<crate::backend::udev::UdevData>,
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

        // Seat：键盘、指针、触摸设备的逻辑组合
        let mut seat_state = SeatState::new();
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, seat_name);

        // 键盘：默认使用 Dvorak 布局
        let xkb_config = smithay::input::keyboard::XkbConfig {
            layout: "us",
            variant: "dvorak",
            ..Default::default()
        };
        seat.add_keyboard(xkb_config, 200, 25).unwrap();
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

            tiling,
            window_map: HashMap::new(),
            next_window_id: 1,
            output_node: None,

            loop_handle,

            config,
            ipc_server,

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
        let output_id = workspace::add_output(
            &mut self.tiling,
            name,
            Rect::new(0, 0, width, height),
        );
        self.output_node = Some(output_id);

        // 创建默认工作区 "1"
        workspace::add_workspace(&mut self.tiling, output_id, "1");
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
        let gaps = rway_tiling::GapsConfig {
            inner: self.config.gaps.inner as i32,
            outer: self.config.gaps.outer as i32,
        };
        layout::compute_layout(&mut self.tiling, root, available, &gaps);

        // 获取所有窗口的几何并更新 Space
        let geometries = layout::get_window_geometries(&self.tiling);
        for (window_id, rect) in geometries {
            if let Some(window) = self.window_map.get(&window_id) {
                // 配置窗口大小
                if let Some(toplevel) = window.toplevel() {
                    toplevel.with_pending_state(|state| {
                        state.size = Some((rect.width, rect.height).into());
                    });
                    toplevel.send_pending_configure();
                }
                // 更新窗口在 Space 中的位置
                self.space.map_element(window.clone(), (rect.x, rect.y), false);
            }
        }
    }

    /// 检测并清理已关闭的窗口：从平铺树和 window_map 中移除死亡窗口
    ///
    /// 应在每帧 `space.refresh()` 之后调用。
    pub fn cleanup_dead_windows(&mut self) {
        // 收集已死亡的窗口 ID（toplevel 不再存在的窗口视为已关闭）
        let dead_ids: Vec<u64> = self
            .window_map
            .iter()
            .filter(|(_, window)| window.toplevel().is_none())
            .map(|(id, _)| *id)
            .collect();

        if dead_ids.is_empty() {
            return;
        }

        // 从平铺树和 window_map 中移除
        for id in &dead_ids {
            rway_tiling::commands::remove_window(&mut self.tiling, *id);
            self.window_map.remove(id);
        }

        tracing::debug!("清理了 {} 个已关闭窗口", dead_ids.len());

        // 重新布局以填充空出的空间
        self.relayout();
    }

    /// 分配下一个窗口 ID
    pub fn alloc_window_id(&mut self) -> u64 {
        let id = self.next_window_id;
        self.next_window_id += 1;
        id
    }

    /// 加载配置文件
    pub fn load_config() -> rway_config::Config {
        // 依次尝试: $XDG_CONFIG_HOME/rway/config -> ~/.config/rway/config -> ~/.config/sway/config
        let config_paths = [
            dirs_config_path("rway/config"),
            dirs_config_path("sway/config"),
        ];

        for path in config_paths.into_iter().flatten() {
            if path.exists() {
                match rway_config::parse_file(&path) {
                    Ok(mut config) => {
                        tracing::info!("已加载配置: {:?}", path);
                        // 如果配置文件没有快捷键（比如使用了不支持的 include 指令），
                        // 合入默认快捷键以保证基本可用性
                        if config.keybindings.is_empty() {
                            tracing::warn!(
                                "配置文件未定义任何快捷键（可能使用了 include 指令，暂不支持），使用内置默认快捷键"
                            );
                            config.keybindings = rway_config::Config::with_defaults().keybindings;
                        }
                        tracing::info!("已加载 {} 个快捷键", config.keybindings.len());
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("配置解析失败 {:?}: {}", path, e);
                    }
                }
            }
        }

        tracing::info!("使用默认配置（含内置快捷键）");
        rway_config::Config::with_defaults()
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

    /// 初始化 Wayland 套接字监听源，注册到事件循环
    fn init_wayland_listener(display: Display<RwayState>, event_loop: &mut EventLoop<Self>) -> OsString {
        let listening_socket = ListeningSocketSource::new_auto().unwrap();
        let socket_name = listening_socket.socket_name().to_os_string();

        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("初始化 Wayland 事件源失败");

        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    unsafe {
                        display.get_mut().dispatch_clients(state).unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }

    /// 在给定逻辑坐标 `pos` 下，找到对应的 WlSurface 及其相对坐标
    pub fn surface_under(&self, pos: Point<f64, Logical>) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.space.element_under(pos).and_then(|(window, location)| {
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
