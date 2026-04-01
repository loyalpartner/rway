#![allow(irrefutable_let_patterns)]

mod backend;
mod border;
mod focus;
mod grabs;
mod handlers;
mod input;
mod ipc;
mod render;
mod shell;
mod state;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
pub use state::RwayState;

/// 检测当前是否运行在 TTY 上（没有宿主 Wayland 合成器）
fn is_on_tty() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_err()
}

/// 解析命令行参数中的后端选择
fn parse_backend_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--backend" {
            return args.get(i + 1).cloned();
        }
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    tracing::info!("启动 rway 合成器");

    // 选择后端：--backend winit|udev，或自动检测
    let backend = parse_backend_arg().unwrap_or_else(|| {
        if is_on_tty() {
            "udev".to_string()
        } else {
            "winit".to_string()
        }
    });

    match backend.as_str() {
        #[cfg(feature = "udev")]
        "udev" => {
            tracing::info!("使用 udev（DRM/KMS）后端");
            backend::udev::run_udev();
        }
        "winit" => {
            tracing::info!("使用 winit 后端");
            run_winit()?;
        }
        other => {
            tracing::error!("未知后端: {}（支持: winit, udev）", other);
            return Err(format!("未知后端: {}", other).into());
        }
    }

    Ok(())
}

/// Winit 后端启动流程（原始的 main 逻辑）
fn run_winit() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 calloop 事件循环（负责调度所有 I/O 和定时器事件）
    let mut event_loop: EventLoop<RwayState> = EventLoop::try_new()?;

    // 创建 Wayland Display（管理所有客户端连接和协议对象）
    let display: Display<RwayState> = Display::new()?;

    // 初始化合成器核心状态
    let mut state = RwayState::new(&mut event_loop, display);

    // 初始化 winit 后端（打开宿主窗口作为输出）
    crate::backend::winit::init_winit(&mut event_loop, &mut state)?;

    // 注册 IPC 轮询到事件循环
    ipc::register_ipc_source(&event_loop.handle());

    // 设置 WAYLAND_DISPLAY 环境变量，使子进程自动连接到本合成器
    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);
    tracing::info!("Wayland 套接字：{:?}", state.socket_name);

    // 可选：根据命令行参数启动客户端
    spawn_client();

    // 运行事件循环直到收到停止信号
    event_loop.run(None, &mut state, move |_| {
        // 每轮分发后的空闲回调（暂无操作）
    })?;

    Ok(())
}

fn init_logging() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }
}

/// 根据 `-c <命令>` 参数启动初始客户端，默认尝试 weston-terminal
pub fn spawn_client() {
    let mut args = std::env::args().skip(1);
    let flag = args.next();
    let arg = args.next();

    match (flag.as_deref(), arg) {
        (Some("-c") | Some("--command"), Some(command)) => {
            std::process::Command::new(command).spawn().ok();
        }
        _ => {
            std::process::Command::new("weston-terminal").spawn().ok();
        }
    }
}
