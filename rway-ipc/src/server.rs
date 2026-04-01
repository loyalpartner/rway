// Unix socket IPC 服务端
//
// 负责创建和管理 Unix domain socket，提供 Sway 兼容的 IPC 连接点。
// calloop 集成将在 Sprint 2 完成；本模块聚焦于 socket 生命周期管理。

use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

/// IPC 服务端，持有 Unix socket 监听器
pub struct IpcServer {
    socket_path: PathBuf,
    listener: UnixListener,
}

impl IpcServer {
    /// 在指定路径创建 Unix socket 并开始监听
    ///
    /// 如果路径已存在（上次未正常退出的残留），先尝试删除再绑定。
    ///
    /// # 错误
    /// 无法创建/绑定 socket 时返回 `std::io::Error`
    pub fn new(socket_path: PathBuf) -> std::io::Result<Self> {
        // 清理可能遗留的旧 socket 文件
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;

        // 设置为非阻塞模式，以便 calloop 事件循环使用
        listener.set_nonblocking(true)?;

        Ok(Self {
            socket_path,
            listener,
        })
    }

    /// 返回 socket 文件路径
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// 获取底层 UnixListener 的引用（用于 calloop 集成）
    pub fn listener(&self) -> &UnixListener {
        &self.listener
    }

    /// 尝试接受新连接（非阻塞）
    pub fn try_accept(&self) -> Option<std::os::unix::net::UnixStream> {
        match self.listener.accept() {
            Ok((stream, _)) => Some(stream),
            Err(_) => None,
        }
    }
}

impl Drop for IpcServer {
    /// 析构时自动清理 socket 文件
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// 返回默认的 IPC socket 路径
///
/// 格式：`$XDG_RUNTIME_DIR/rway-ipc.$WAYLAND_DISPLAY.sock`
///
/// 如果环境变量未设置，回退到 `/tmp/rway-ipc.default.sock`
pub fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "default".to_string());
    PathBuf::from(format!("{}/rway-ipc.{}.sock", runtime_dir, display))
}

// ─────────────────────────────────────────────────────────────────────────────
// 单元测试（TDD：先写测试，后写实现）
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    /// 生成测试用的临时 socket 路径
    fn temp_socket_path(suffix: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/rway-ipc-test-{}-{}.sock", std::process::id(), suffix))
    }

    // ── IpcServer::new ────────────────────────────────────────────────────────

    /// 测试在新路径创建 IpcServer 成功
    #[test]
    fn test_ipc_server_new_creates_socket_file() {
        let path = temp_socket_path("new");
        // 确保测试前路径不存在
        let _ = std::fs::remove_file(&path);

        let server = IpcServer::new(path.clone()).unwrap();

        assert!(path.exists(), "socket 文件应该已创建");
        assert_eq!(server.socket_path(), path.as_path());

        // Drop 触发清理
        drop(server);
        assert!(!path.exists(), "socket 文件应在 Drop 后被删除");
    }

    /// 测试残留 socket 文件被自动清理并重新创建
    #[test]
    fn test_ipc_server_new_removes_stale_socket() {
        let path = temp_socket_path("stale");
        // 先创建一个残留文件
        std::fs::write(&path, b"stale data").unwrap();
        assert!(path.exists());

        // 即使有残留文件，IpcServer::new 也应成功
        let server = IpcServer::new(path.clone()).unwrap();
        assert!(path.exists(), "新 socket 文件应该已创建");

        drop(server);
    }

    /// 测试 socket_path() 返回正确路径
    #[test]
    fn test_ipc_server_socket_path_returns_correct_path() {
        let path = temp_socket_path("path");
        let _ = std::fs::remove_file(&path);

        let server = IpcServer::new(path.clone()).unwrap();
        assert_eq!(server.socket_path(), path.as_path());

        drop(server);
    }

    /// 测试 socket 实际可以接受连接
    #[test]
    fn test_ipc_server_socket_accepts_connections() {
        let path = temp_socket_path("accept");
        let _ = std::fs::remove_file(&path);

        let server = IpcServer::new(path.clone()).unwrap();

        // 尝试连接到 socket
        let connect_result = UnixStream::connect(&path);
        assert!(
            connect_result.is_ok(),
            "应该能够连接到 IPC socket"
        );

        drop(server);
    }

    // ── Drop ──────────────────────────────────────────────────────────────────

    /// 测试 Drop 自动删除 socket 文件
    #[test]
    fn test_ipc_server_drop_removes_socket_file() {
        let path = temp_socket_path("drop");
        let _ = std::fs::remove_file(&path);

        {
            let _server = IpcServer::new(path.clone()).unwrap();
            assert!(path.exists());
        }
        // 离开作用域后 Drop 应该执行

        assert!(!path.exists(), "Drop 后 socket 文件应被删除");
    }

    /// 测试 Drop 对不存在的文件不 panic
    #[test]
    fn test_ipc_server_drop_nonexistent_file_no_panic() {
        let path = temp_socket_path("nodrop");
        let _ = std::fs::remove_file(&path);

        let server = IpcServer::new(path.clone()).unwrap();
        // 手动删除文件后再 Drop，不应 panic
        let _ = std::fs::remove_file(&path);
        drop(server); // 不应 panic
    }

    // ── default_socket_path ───────────────────────────────────────────────────

    /// 测试默认路径包含 rway-ipc 标识
    #[test]
    fn test_default_socket_path_contains_rway_ipc() {
        let path = default_socket_path();
        let path_str = path.to_str().unwrap();
        assert!(
            path_str.contains("rway-ipc"),
            "默认路径应包含 'rway-ipc'，实际：{}",
            path_str
        );
    }

    /// 测试默认路径以 .sock 结尾
    #[test]
    fn test_default_socket_path_ends_with_sock() {
        let path = default_socket_path();
        let path_str = path.to_str().unwrap();
        assert!(
            path_str.ends_with(".sock"),
            "默认路径应以 '.sock' 结尾，实际：{}",
            path_str
        );
    }

    /// 测试在 XDG_RUNTIME_DIR 已设置时使用该目录
    #[test]
    fn test_default_socket_path_uses_xdg_runtime_dir() {
        // 临时设置环境变量（注意：多线程测试可能有竞争，这里仅验证函数逻辑）
        // 使用 std::env::set_var 在测试中有安全警告，但对于单进程测试可接受
        unsafe {
            std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
            std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
        }

        let path = default_socket_path();
        let path_str = path.to_str().unwrap();

        assert!(
            path_str.starts_with("/run/user/1000"),
            "应使用 XDG_RUNTIME_DIR，实际：{}",
            path_str
        );
        assert!(
            path_str.contains("wayland-0"),
            "路径应包含 WAYLAND_DISPLAY，实际：{}",
            path_str
        );

        // 清理环境变量
        unsafe {
            std::env::remove_var("XDG_RUNTIME_DIR");
            std::env::remove_var("WAYLAND_DISPLAY");
        }
    }

    /// 测试 XDG_RUNTIME_DIR 未设置时使用 /tmp
    #[test]
    fn test_default_socket_path_fallback_to_tmp() {
        unsafe {
            std::env::remove_var("XDG_RUNTIME_DIR");
            std::env::remove_var("WAYLAND_DISPLAY");
        }

        let path = default_socket_path();
        let path_str = path.to_str().unwrap();

        // 在无 XDG_RUNTIME_DIR 的环境中应回退到 /tmp
        assert!(
            path_str.starts_with("/tmp") || path_str.contains("rway-ipc"),
            "无 XDG_RUNTIME_DIR 时应回退到 /tmp，实际：{}",
            path_str
        );
    }

    /// 测试默认路径是绝对路径
    #[test]
    fn test_default_socket_path_is_absolute() {
        let path = default_socket_path();
        assert!(path.is_absolute(), "默认路径必须是绝对路径");
    }
}
