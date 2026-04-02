// backend/udev.rs — DRM/KMS 原生后端（直接在 TTY 上运行）
//
// 允许 rway 无需宿主合成器，直接通过 DRM/KMS 驱动显示器。
// 参考实现：smithay/anvil/src/udev.rs

use std::{
    collections::HashMap,
    io,
    ops::Not,
    path::Path,
    sync::Once,
    time::{Duration, Instant},
};

use smithay::{
    backend::{
        allocator::{
            dmabuf::Dmabuf,
            format::FormatSet,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc, Modifier,
        },
        drm::{
            compositor::FrameFlags,
            exporter::gbm::GbmFramebufferExporter,
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
            CreateDrmNodeError, DrmAccessError, DrmDevice, DrmDeviceFd, DrmError, DrmEvent,
            DrmEventMetadata, DrmEventTime, DrmNode, NodeType,
        },
        egl::{context::ContextPriority, EGLContext, EGLDevice, EGLDisplay},
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            gles::GlesRenderer,
            multigpu::{gbm::GbmGlesBackend, GpuManager, MultiRenderer},
            ImportDma, ImportEgl, ImportMemWl,
        },
        session::{
            libseat::{self, LibSeatSession},
            Event as SessionEvent, Session,
        },
        udev::{all_gpus, primary_gpu, UdevBackend, UdevEvent},
        SwapBuffersError,
    },
    delegate_dmabuf,
    output::{Mode as WlMode, Output, PhysicalProperties},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop, LoopHandle, RegistrationToken,
        },
        drm::control::{connector, crtc, ModeTypeFlags},
        input::{DeviceCapability, Libinput},
        rustix::fs::OFlags,
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
        wayland_server::{backend::GlobalId, Display, DisplayHandle},
    },
    utils::{DeviceFd, Monotonic, Scale, Time},
    wayland::dmabuf::{
        DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier,
    },
};
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner};
use tracing::{debug, error, info, trace, warn};

use crate::state::RwayState;

// 支持的颜色格式：优先 10-bit，退而求其次 8-bit
const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr2101010,
    Fourcc::Argb2101010,
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
];
const SUPPORTED_FORMATS_8BIT_ONLY: &[Fourcc] = &[Fourcc::Abgr8888, Fourcc::Argb8888];

/// 多 GPU 渲染器类型别名
type UdevRenderer<'a> = MultiRenderer<
    'a,
    'a,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
    GbmGlesBackend<GlesRenderer, DrmDeviceFd>,
>;

smithay::backend::renderer::element::render_elements! {
    pub UdevRenderElement<='a, UdevRenderer<'a>>;
    Space=smithay::desktop::space::SpaceRenderElements<UdevRenderer<'a>, smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement<UdevRenderer<'a>>>,
    Cursor=smithay::backend::renderer::element::solid::SolidColorRenderElement,
}

/// 用于将 Output 与 DRM 设备/CRTC 关联的标识
#[derive(Debug, PartialEq)]
struct UdevOutputId {
    device_id: DrmNode,
    crtc: crtc::Handle,
}

/// Udev backend data structure
pub(crate) struct UdevData {
    pub(crate) session: LibSeatSession,
    _dh: DisplayHandle,
    dmabuf_state: Option<(DmabufState, DmabufGlobal)>,
    primary_gpu: DrmNode,
    gpus: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    backends: HashMap<DrmNode, BackendData>,
    keyboards: Vec<smithay::reexports::input::Device>,
}

/// 每个 DRM 设备的状态
struct BackendData {
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmFramebufferExporter<DrmDeviceFd>,
        Option<()>,
        DrmDeviceFd,
    >,
    drm_scanner: DrmScanner,
    render_node: Option<DrmNode>,
    registration_token: RegistrationToken,
}

/// 每个 DRM 输出表面（连接器+CRTC）的状态
struct SurfaceData {
    dh: DisplayHandle,
    device_id: DrmNode,
    render_node: Option<DrmNode>,
    output: Output,
    global: Option<GlobalId>,
    drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmFramebufferExporter<DrmDeviceFd>,
        Option<()>,
        DrmDeviceFd,
    >,
    last_presentation_time: Option<Time<Monotonic>>,
    vblank_throttle_timer: Option<RegistrationToken>,
}

impl Drop for SurfaceData {
    fn drop(&mut self) {
        // 清理输出关联的客户端资源
        if let Some(global) = self.global.take() {
            self.dh.remove_global::<RwayState>(global);
        }
    }
}

/// 设备添加时可能出现的错误
#[derive(Debug, thiserror::Error)]
enum DeviceAddError {
    #[error("通过 libseat 打开设备失败: {0}")]
    DeviceOpen(libseat::Error),
    #[error("初始化 DRM 设备失败: {0}")]
    DrmDevice(DrmError),
    #[error("初始化 GBM 设备失败: {0}")]
    GbmDevice(std::io::Error),
    #[error("访问 DRM 节点失败: {0}")]
    DrmNode(CreateDrmNodeError),
    #[error("添加设备到 GpuManager 失败: {0}")]
    AddNode(smithay::backend::egl::Error),
    #[error("获取渲染器失败")]
    GpuManager,
    #[error("设备没有渲染节点")]
    NoRenderNode,
    #[error("主 GPU 缺失")]
    PrimaryGpuMissing,
}

/// DmabufHandler 实现：处理客户端导入的 DMA-BUF
impl DmabufHandler for RwayState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self
            .udev_data
            .as_mut()
            .expect("udev backend initialized")
            .dmabuf_state
            .as_mut()
            .expect("udev init: dmabuf state not initialized")
            .0
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        let udev = self.udev_data.as_mut().expect("udev backend initialized");
        if udev
            .gpus
            .single_renderer(&udev.primary_gpu)
            .and_then(|mut renderer| renderer.import_dmabuf(&dmabuf, None))
            .is_ok()
        {
            dmabuf.set_node(udev.primary_gpu);
            let _ = notifier.successful::<RwayState>();
        } else {
            notifier.failed();
        }
    }
}
delegate_dmabuf!(RwayState);

/// Udev backend entry point
pub(crate) fn run_udev() {
    let mut event_loop: EventLoop<RwayState> =
        EventLoop::try_new().expect("Failed to create event loop");
    let display: Display<RwayState> = Display::new().expect("Failed to create Wayland display");
    let display_handle = display.handle();

    // ── 1. 初始化 libseat 会话 ──
    let (session, notifier) = match LibSeatSession::new() {
        Ok(ret) => ret,
        Err(err) => {
            error!("无法初始化会话: {}", err);
            return;
        }
    };

    let seat_name = session.seat();
    info!("会话已初始化，seat: {}", seat_name);

    // ── 2. 查找主 GPU ──
    let primary_gpu = if let Ok(var) = std::env::var("RWAY_DRM_DEVICE") {
        DrmNode::from_path(var).expect("无效的 DRM 设备路径")
    } else {
        primary_gpu(&seat_name)
            .expect("udev init: failed to query primary GPU")
            .and_then(|x| {
                DrmNode::from_path(x)
                    .ok()?
                    .node_with_type(NodeType::Render)?
                    .ok()
            })
            .unwrap_or_else(|| {
                all_gpus(&seat_name)
                    .expect("udev init: failed to enumerate GPUs")
                    .into_iter()
                    .find_map(|x| DrmNode::from_path(x).ok())
                    .expect("udev init: no GPU found!")
            })
    };
    info!("使用 {} 作为主 GPU", primary_gpu);

    // ── 3. 创建 GpuManager ──
    let gpus = GpuManager::new(GbmGlesBackend::with_factory(|display: &EGLDisplay| {
        let context = EGLContext::new_with_priority(display, ContextPriority::High)?;
        let capabilities = unsafe { GlesRenderer::supported_capabilities(&context)? };
        Ok(unsafe { GlesRenderer::with_capabilities(context, capabilities)? })
    }))
    .expect("udev init: failed to create GpuManager");

    // ── 4. 创建 UdevData ──
    let udev_data = UdevData {
        _dh: display_handle.clone(),
        dmabuf_state: None,
        session,
        primary_gpu,
        gpus,
        backends: HashMap::new(),
        keyboards: Vec::new(),
    };

    // ── 5. 创建 RwayState（使用 seat 名称） ──
    let mut state = RwayState::new_with_seat_name(&mut event_loop, display, &seat_name);
    state.udev_data = Some(udev_data);

    let handle = event_loop.handle();

    // ── 6. 初始化 udev 后端（扫描设备） ──
    let udev_backend = match UdevBackend::new(&seat_name) {
        Ok(ret) => ret,
        Err(err) => {
            error!("初始化 udev 后端失败: {:?}", err);
            return;
        }
    };

    // ── 7. 初始化 libinput ──
    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        state
            .udev_data
            .as_ref()
            .expect("udev backend initialized")
            .session
            .clone()
            .into(),
    );
    libinput_context
        .udev_assign_seat(&seat_name)
        .expect("udev init: failed to assign seat to libinput");
    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    // 注册 libinput 事件源
    handle
        .insert_source(
            libinput_backend,
            move |mut event, _, state: &mut RwayState| {
                if let InputEvent::DeviceAdded { device } = &mut event {
                    if device.has_capability(DeviceCapability::Keyboard) {
                        if let Some(led_state) = state.seat.get_keyboard().map(|kb| kb.led_state())
                        {
                            device.led_update(led_state.into());
                        }
                        state
                            .udev_data
                            .as_mut()
                            .expect("udev backend initialized")
                            .keyboards
                            .push(device.clone());
                    }
                } else if let InputEvent::DeviceRemoved { ref device } = event {
                    if device.has_capability(DeviceCapability::Keyboard) {
                        state
                            .udev_data
                            .as_mut()
                            .expect("udev backend initialized")
                            .keyboards
                            .retain(|item| item != device);
                    }
                }

                state.process_input_event(event);
            },
        )
        .expect("udev init: failed to register libinput event source");

    // ── 8. 注册会话通知（VT 切换） ──
    handle
        .insert_source(notifier, move |event, &mut (), state: &mut RwayState| {
            match event {
                SessionEvent::PauseSession => {
                    libinput_context.suspend();
                    info!("会话已暂停");

                    let udev = state.udev_data.as_mut().expect("udev backend initialized");
                    for backend in udev.backends.values_mut() {
                        backend.drm_output_manager.pause();
                    }
                }
                SessionEvent::ActivateSession => {
                    info!("会话已恢复");

                    if let Err(err) = libinput_context.resume() {
                        error!("恢复 libinput 上下文失败: {:?}", err);
                    }
                    let udev = state.udev_data.as_mut().expect("udev backend initialized");
                    for (_node, backend) in udev.backends.iter_mut() {
                        if let Err(err) = backend.drm_output_manager.device_mut().activate(false) {
                            error!("激活 DRM 后端失败: {:?}", err);
                        }
                    }

                    // 恢复后重新渲染所有输出
                    let nodes: Vec<DrmNode> = state
                        .udev_data
                        .as_ref()
                        .expect("udev backend initialized")
                        .backends
                        .keys()
                        .copied()
                        .collect();
                    for node in nodes {
                        render_all_surfaces(state, node);
                    }
                }
            }
        })
        .expect("udev init: failed to register session notifier");

    // ── 9. 扫描设备：优先初始化主 GPU ──
    let primary_node = primary_gpu
        .node_with_type(NodeType::Primary)
        .and_then(|node| node.ok());
    let primary_device = udev_backend.device_list().find(|(device_id, _)| {
        primary_node
            .map(|pn| *device_id == pn.dev_id())
            .unwrap_or(false)
            || *device_id == primary_gpu.dev_id()
    });

    if let Some((device_id, path)) = primary_device {
        let node = DrmNode::from_dev_id(device_id).expect("无法获取主节点");
        if let Err(err) = device_added(&mut state, &handle, node, path) {
            error!("初始化主设备失败: {}", err);
            return;
        }
    }

    let primary_device_id = primary_device.map(|(id, _)| id);
    for (device_id, path) in udev_backend.device_list() {
        if Some(device_id) == primary_device_id {
            continue;
        }
        if let Err(err) = DrmNode::from_dev_id(device_id)
            .map_err(DeviceAddError::DrmNode)
            .and_then(|node| device_added(&mut state, &handle, node, path))
        {
            error!("跳过设备 {device_id}: {err}");
        }
    }

    // 更新 SHM 格式
    {
        let udev = state.udev_data.as_mut().expect("udev backend initialized");
        let shm_formats = udev
            .gpus
            .single_renderer(&primary_gpu)
            .expect("udev init: failed to get renderer for SHM formats")
            .shm_formats();
        state.shm_state.update_formats(shm_formats);
    }

    // ── 10. 初始化 EGL 硬件加速 ──
    {
        let udev = state.udev_data.as_mut().expect("udev backend initialized");
        let mut renderer = udev
            .gpus
            .single_renderer(&primary_gpu)
            .expect("udev init: failed to get renderer for EGL");
        info!("尝试初始化 EGL 硬件加速");
        match renderer.bind_wl_display(&display_handle) {
            Ok(_) => info!("EGL 硬件加速已启用"),
            Err(err) => info!("EGL 硬件加速初始化失败: {:?}", err),
        }
    }

    // ── 11. 初始化 DMA-BUF 支持 ──
    {
        let udev = state.udev_data.as_mut().expect("udev backend initialized");
        let renderer = udev
            .gpus
            .single_renderer(&primary_gpu)
            .expect("udev init: failed to get renderer for DMA-BUF");
        let dmabuf_formats = renderer.dmabuf_formats();
        let default_feedback = DmabufFeedbackBuilder::new(primary_gpu.dev_id(), dmabuf_formats)
            .build()
            .expect("udev init: failed to build DMA-BUF feedback");
        let mut dmabuf_state = DmabufState::new();
        let global = dmabuf_state
            .create_global_with_default_feedback::<RwayState>(&display_handle, &default_feedback);
        udev.dmabuf_state = Some((dmabuf_state, global));
    }

    // ── 12. 注册 udev 热插拔事件源 ──
    handle
        .insert_source(
            udev_backend,
            move |event, _, state: &mut RwayState| match event {
                UdevEvent::Added { device_id, path } => {
                    let handle = state.loop_handle.clone();
                    if let Err(err) = DrmNode::from_dev_id(device_id)
                        .map_err(DeviceAddError::DrmNode)
                        .and_then(|node| device_added(state, &handle, node, &path))
                    {
                        error!("跳过设备 {device_id}: {err}");
                    }
                }
                UdevEvent::Changed { device_id } => {
                    if let Ok(node) = DrmNode::from_dev_id(device_id) {
                        device_changed(state, node);
                    }
                }
                UdevEvent::Removed { device_id } => {
                    if let Ok(node) = DrmNode::from_dev_id(device_id) {
                        device_removed(state, node);
                    }
                }
            },
        )
        .expect("udev init: failed to register udev hotplug event source");

    // 注册 IPC 轮询
    crate::ipc::register_ipc_source(&event_loop.handle());

    // 设置环境变量
    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);
    info!("Wayland 套接字：{:?}", state.socket_name);

    // 启动 XWayland 以支持 X11 应用
    #[cfg(feature = "xwayland")]
    state.start_xwayland();

    // 启动初始客户端
    crate::spawn_client();

    // ── 13. 运行事件循环 ──
    info!("进入主事件循环");
    event_loop
        .run(None, &mut state, move |state| {
            // 推进动画插值，更新窗口在 Space 中的渲染位置
            state.update_animations();
            state.space.refresh();
            state.popups.cleanup();
            state.cleanup_dead_windows();
            let _ = state.display_handle.flush_clients();
        })
        .expect("udev: event loop terminated with error");
}

// ────────────────────────────────────────────────────────────────
// 设备管理
// ────────────────────────────────────────────────────────────────

/// 添加新的 DRM 设备
fn device_added(
    state: &mut RwayState,
    handle: &LoopHandle<'static, RwayState>,
    node: DrmNode,
    path: &Path,
) -> Result<(), DeviceAddError> {
    let udev = state.udev_data.as_mut().expect("udev backend initialized");

    // 通过 libseat 打开设备文件
    let fd = udev
        .session
        .open(
            path,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
        )
        .map_err(DeviceAddError::DeviceOpen)?;

    let fd = DrmDeviceFd::new(DeviceFd::from(fd));
    let (drm, drm_notifier) =
        DrmDevice::new(fd.clone(), true).map_err(DeviceAddError::DrmDevice)?;
    let gbm = GbmDevice::new(fd).map_err(DeviceAddError::GbmDevice)?;

    // 注册 VBlank 事件源
    let registration_token = handle
        .insert_source(
            drm_notifier,
            move |event, metadata, state: &mut RwayState| match event {
                DrmEvent::VBlank(crtc) => {
                    frame_finish(state, node, crtc, metadata);
                }
                DrmEvent::Error(error) => {
                    error!("DRM 错误: {:?}", error);
                }
            },
        )
        .expect("udev init: failed to register DRM VBlank event source");

    // 尝试初始化 GPU（获取 EGL 渲染节点）
    let render_node = {
        let mut try_init = || -> Result<DrmNode, DeviceAddError> {
            let display = unsafe { EGLDisplay::new(gbm.clone()).map_err(DeviceAddError::AddNode)? };
            let egl_device =
                EGLDevice::device_for_display(&display).map_err(DeviceAddError::AddNode)?;
            if egl_device.is_software() {
                return Err(DeviceAddError::NoRenderNode);
            }
            let rn = egl_device
                .try_get_render_node()
                .ok()
                .flatten()
                .unwrap_or(node);
            udev.gpus
                .as_mut()
                .add_node(rn, gbm.clone())
                .map_err(DeviceAddError::AddNode)?;
            Ok(rn)
        };

        try_init()
            .inspect_err(|err| warn!("GPU 初始化失败: {:?}", err))
            .ok()
    };

    // 创建分配器
    let allocator = render_node
        .is_some()
        .then(|| {
            GbmAllocator::new(
                gbm.clone(),
                GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
            )
        })
        .or_else(|| {
            udev.backends
                .get(&udev.primary_gpu)
                .map(|b| b.drm_output_manager.allocator().clone())
        })
        .ok_or(DeviceAddError::PrimaryGpuMissing)?;

    let framebuffer_exporter = GbmFramebufferExporter::new(gbm.clone(), render_node);

    let color_formats = if std::env::var("RWAY_DISABLE_10BIT").is_ok() {
        SUPPORTED_FORMATS_8BIT_ONLY
    } else {
        SUPPORTED_FORMATS
    };

    // 获取渲染器支持的格式
    let render_node_for_renderer = render_node.unwrap_or(udev.primary_gpu);
    let mut renderer = match udev.gpus.single_renderer(&render_node_for_renderer) {
        Ok(r) => r,
        Err(err) => {
            error!("获取渲染器失败: {:?}", err);
            return Err(DeviceAddError::GpuManager);
        }
    };
    let render_formats = renderer
        .as_mut()
        .egl_context()
        .dmabuf_render_formats()
        .iter()
        .filter(|format| render_node.is_some() || format.modifier == Modifier::Linear)
        .copied()
        .collect::<FormatSet>();

    let drm_output_manager = DrmOutputManager::new(
        drm,
        allocator,
        framebuffer_exporter,
        Some(gbm),
        color_formats.iter().copied(),
        render_formats,
    );

    udev.backends.insert(
        node,
        BackendData {
            registration_token,
            drm_output_manager,
            drm_scanner: DrmScanner::new(),
            render_node,
            surfaces: HashMap::new(),
        },
    );

    // 扫描已连接的连接器
    device_changed(state, node);

    Ok(())
}

/// 设备连接器变更（热插拔）
fn device_changed(state: &mut RwayState, node: DrmNode) {
    let udev = state.udev_data.as_mut().expect("udev backend initialized");
    let device = match udev.backends.get_mut(&node) {
        Some(d) => d,
        None => return,
    };

    let scan_result = match device
        .drm_scanner
        .scan_connectors(device.drm_output_manager.device())
    {
        Ok(result) => result,
        Err(err) => {
            warn!("扫描连接器失败: {:?}", err);
            return;
        }
    };

    for event in scan_result {
        match event {
            DrmScanEvent::Connected {
                connector,
                crtc: Some(crtc),
            } => {
                connector_connected(state, node, connector, crtc);
            }
            DrmScanEvent::Disconnected {
                connector,
                crtc: Some(crtc),
            } => {
                connector_disconnected(state, node, connector, crtc);
            }
            _ => {}
        }
    }
}

/// 设备已移除
fn device_removed(state: &mut RwayState, node: DrmNode) {
    // 先收集所有需要断开的连接器，避免借用冲突
    let crtcs: Vec<_> = {
        let udev = state.udev_data.as_mut().expect("udev backend initialized");
        let device = match udev.backends.get_mut(&node) {
            Some(d) => d,
            None => return,
        };
        device
            .drm_scanner
            .crtcs()
            .map(|(info, crtc)| (info.clone(), crtc))
            .collect()
    };

    // 断开所有连接器（此时无 udev 借用）
    for (connector, crtc) in crtcs {
        connector_disconnected(state, node, connector, crtc);
    }

    debug!("所有表面已释放");

    let udev = state.udev_data.as_mut().expect("udev backend initialized");
    if let Some(backend_data) = udev.backends.remove(&node) {
        if let Some(render_node) = backend_data.render_node {
            udev.gpus.as_mut().remove_node(&render_node);
        }
        state.loop_handle.remove(backend_data.registration_token);
        debug!("设备已释放");
    }
}

/// 连接器已连接：创建输出和 DRM 表面
fn connector_connected(
    state: &mut RwayState,
    node: DrmNode,
    connector: connector::Info,
    crtc: crtc::Handle,
) {
    let udev = state.udev_data.as_mut().expect("udev backend initialized");
    let device = match udev.backends.get_mut(&node) {
        Some(d) => d,
        None => return,
    };

    let render_node = device.render_node.unwrap_or(udev.primary_gpu);
    let mut renderer = match udev.gpus.single_renderer(&render_node) {
        Ok(r) => r,
        Err(err) => {
            warn!("获取渲染器失败，跳过连接器: {:?}", err);
            return;
        }
    };

    let output_name = format!(
        "{}-{}",
        connector.interface().as_str(),
        connector.interface_id()
    );
    info!(crtc = ?crtc, "正在设置连接器 {}", output_name);

    let _drm_device = device.drm_output_manager.device();

    // 显示器信息（暂时使用默认值，需要 libdisplay-info 获取 EDID 数据）
    let make = "Unknown".to_string();
    let model = "Unknown".to_string();

    // 选择首选模式（PREFERRED 优先）
    let mode_id = connector
        .modes()
        .iter()
        .position(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
        .unwrap_or(0);
    let drm_mode = connector.modes()[mode_id];
    let wl_mode = WlMode::from(drm_mode);

    // 创建 Wayland 输出
    let (phys_w, phys_h) = connector.size().unwrap_or((0, 0));
    let output = Output::new(
        output_name.clone(),
        PhysicalProperties {
            size: (phys_w as i32, phys_h as i32).into(),
            subpixel: connector.subpixel().into(),
            make,
            model,
        },
    );
    let global = output.create_global::<RwayState>(&state.display_handle);

    // 计算输出位置（水平排列）
    let x = state.space.outputs().fold(0, |acc, o| {
        acc + state
            .space
            .output_geometry(o)
            .map(|g| g.size.w)
            .unwrap_or(0)
    });
    let position = (x, 0).into();

    output.set_preferred(wl_mode);
    output.change_current_state(Some(wl_mode), None, None, Some(position));
    state.space.map_output(&output, position);

    output.user_data().insert_if_missing(|| UdevOutputId {
        crtc,
        device_id: node,
    });

    // 初始化 DRM 输出
    let drm_output = match device
        .drm_output_manager
        .initialize_output::<
            _,
            smithay::backend::renderer::element::solid::SolidColorRenderElement,
        >(
            crtc,
            drm_mode,
            &[connector.handle()],
            &output,
            None,
            &mut renderer,
            &DrmOutputRenderElements::default(),
        ) {
        Ok(drm_output) => drm_output,
        Err(err) => {
            warn!("初始化 DRM 输出失败: {:?}", err);
            return;
        }
    };

    let surface = SurfaceData {
        dh: state.display_handle.clone(),
        device_id: node,
        render_node: device.render_node,
        output: output.clone(),
        global: Some(global),
        drm_output,
        last_presentation_time: None,
        vblank_throttle_timer: None,
    };

    device.surfaces.insert(crtc, surface);

    info!(
        "输出 {} 已就绪（{}x{} @ {}mHz）",
        output_name, wl_mode.size.w, wl_mode.size.h, wl_mode.refresh
    );

    // 初始化平铺树中的输出
    if state.output_node.is_none() {
        state.init_tiling_output(wl_mode.size.w, wl_mode.size.h);
    }

    // 触发首次渲染
    let handle = state.loop_handle.clone();
    handle.insert_idle(move |state| {
        render_surface(state, node, crtc, Time::<Monotonic>::from(Duration::ZERO));
    });
}

/// 连接器已断开
fn connector_disconnected(
    state: &mut RwayState,
    node: DrmNode,
    _connector: connector::Info,
    crtc: crtc::Handle,
) {
    let udev = state.udev_data.as_mut().expect("udev backend initialized");
    let device = match udev.backends.get_mut(&node) {
        Some(d) => d,
        None => return,
    };

    if let Some(surface) = device.surfaces.remove(&crtc) {
        info!("连接器已断开: {}", surface.output.name());
        state.space.unmap_output(&surface.output);
        state.space.refresh();
    }
}

// ────────────────────────────────────────────────────────────────
// 渲染循环
// ────────────────────────────────────────────────────────────────

/// 辅助函数：为设备的所有 CRTC 安排渲染
fn render_all_surfaces(state: &mut RwayState, node: DrmNode) {
    let crtcs: Vec<crtc::Handle> = state
        .udev_data
        .as_ref()
        .expect("udev backend initialized")
        .backends
        .get(&node)
        .map(|b| b.surfaces.keys().copied().collect())
        .unwrap_or_default();

    let handle = state.loop_handle.clone();
    for crtc in crtcs {
        handle.insert_idle(move |state| {
            render_surface(state, node, crtc, Time::<Monotonic>::from(Duration::ZERO));
        });
    }
}

/// 处理 VBlank 事件
fn frame_finish(
    state: &mut RwayState,
    dev_id: DrmNode,
    crtc: crtc::Handle,
    metadata: &mut Option<DrmEventMetadata>,
) {
    let udev = state.udev_data.as_mut().expect("udev backend initialized");
    let device = match udev.backends.get_mut(&dev_id) {
        Some(b) => b,
        None => {
            error!("在不存在的后端 {} 上完成帧", dev_id);
            return;
        }
    };

    let surface = match device.surfaces.get_mut(&crtc) {
        Some(s) => s,
        None => {
            error!("在不存在的 CRTC {:?} 上完成帧", crtc);
            return;
        }
    };

    // 取消节流定时器
    if let Some(timer_token) = surface.vblank_throttle_timer.take() {
        state.loop_handle.remove(timer_token);
    }

    let output = match state.space.outputs().find(|o| {
        o.user_data().get::<UdevOutputId>()
            == Some(&UdevOutputId {
                device_id: surface.device_id,
                crtc,
            })
    }) {
        Some(o) => o.clone(),
        None => return,
    };

    let Some(frame_duration) = output
        .current_mode()
        .map(|mode| Duration::from_secs_f64(1_000f64 / mode.refresh as f64))
    else {
        return;
    };

    let tp = metadata.as_ref().and_then(|m| match m.time {
        DrmEventTime::Monotonic(tp) => tp.is_zero().not().then_some(tp),
        DrmEventTime::Realtime(_) => None,
    });

    let seq = metadata.as_ref().map(|m| m.sequence).unwrap_or(0);

    let (clock, _flags) = if let Some(tp) = tp {
        (
            tp.into(),
            wp_presentation_feedback::Kind::Vsync
                | wp_presentation_feedback::Kind::HwClock
                | wp_presentation_feedback::Kind::HwCompletion,
        )
    } else {
        (
            Time::<Monotonic>::from(Duration::ZERO),
            wp_presentation_feedback::Kind::Vsync,
        )
    };

    // VBlank 节流：防止显示器运行过快
    let Some(udev) = state.udev_data.as_mut() else {
        return;
    };
    let Some(device) = udev.backends.get_mut(&dev_id) else {
        return;
    };
    let Some(surface) = device.surfaces.get_mut(&crtc) else {
        return;
    };

    let vblank_remaining = surface
        .last_presentation_time
        .map(|last| frame_duration.saturating_sub(Time::elapsed(&last, clock)));

    if let Some(remaining) = vblank_remaining {
        if remaining > frame_duration / 2 {
            static WARN_ONCE: Once = Once::new();
            WARN_ONCE.call_once(|| {
                warn!("显示器运行速度超过预期，正在节流 VBlank");
            });
            let throttled_time = tp
                .map(|tp| tp.saturating_add(remaining))
                .unwrap_or(Duration::ZERO);
            let throttled_metadata = DrmEventMetadata {
                sequence: seq,
                time: DrmEventTime::Monotonic(throttled_time),
            };
            let timer_token = state
                .loop_handle
                .insert_source(Timer::from_duration(remaining), move |_, _, state| {
                    frame_finish(state, dev_id, crtc, &mut Some(throttled_metadata));
                    TimeoutAction::Drop
                })
                .expect("注册 VBlank 节流定时器失败");
            if let Some(udev) = state.udev_data.as_mut() {
                if let Some(device) = udev.backends.get_mut(&dev_id) {
                    if let Some(surface) = device.surfaces.get_mut(&crtc) {
                        surface.vblank_throttle_timer = Some(timer_token);
                    }
                }
            }
            return;
        }
    }
    surface.last_presentation_time = Some(clock);

    // 提交帧
    let submit_result = surface
        .drm_output
        .frame_submitted()
        .map_err(Into::<SwapBuffersError>::into);

    let schedule_render = match submit_result {
        Ok(_user_data) => {
            // user_data 是 Option<()>，我们简化不处理 presentation feedback
            true
        }
        Err(err) => {
            warn!("渲染出错: {:?}", err);
            match err {
                SwapBuffersError::AlreadySwapped => true,
                SwapBuffersError::TemporaryFailure(err)
                    if matches!(
                        err.downcast_ref::<DrmError>(),
                        Some(&DrmError::DeviceInactive)
                    ) =>
                {
                    false
                }
                SwapBuffersError::TemporaryFailure(err) => matches!(
                    err.downcast_ref::<DrmError>(),
                    Some(DrmError::Access(DrmAccessError { source, .. }))
                        if source.kind() == io::ErrorKind::PermissionDenied
                ),
                SwapBuffersError::ContextLost(err) => {
                    panic!("渲染循环丢失: {err}")
                }
            }
        }
    };

    if schedule_render {
        let next_frame_target = clock + frame_duration;

        // 延迟重绘以降低客户端缓冲区延迟
        let repaint_delay = Duration::from_secs_f64(frame_duration.as_secs_f64() * 0.6);

        let Some(udev) = state.udev_data.as_ref() else {
            return;
        };
        let Some(device) = udev.backends.get(&dev_id) else {
            return;
        };
        let Some(surface) = device.surfaces.get(&crtc) else {
            return;
        };

        let timer = if surface
            .render_node
            .map(|rn| rn != udev.primary_gpu)
            .unwrap_or(true)
        {
            trace!("在 {:?} 上立即调度重绘", crtc);
            Timer::immediate()
        } else {
            trace!("在 {:?} 上延迟 {:?} 调度重绘", crtc, repaint_delay);
            Timer::from_duration(repaint_delay)
        };

        state
            .loop_handle
            .insert_source(timer, move |_, _, state| {
                render_surface(state, dev_id, crtc, next_frame_target);
                TimeoutAction::Drop
            })
            .expect("调度帧定时器失败");
    }
}

/// 渲染指定 DRM 输出
fn render_surface(
    state: &mut RwayState,
    node: DrmNode,
    crtc: crtc::Handle,
    frame_target: Time<Monotonic>,
) {
    // 查找对应的 Wayland 输出
    let output = {
        let udev = state.udev_data.as_ref().expect("udev backend initialized");
        let device = match udev.backends.get(&node) {
            Some(d) => d,
            None => {
                error!("在不存在的后端 {} 上渲染", node);
                return;
            }
        };
        let surface = match device.surfaces.get(&crtc) {
            Some(s) => s,
            None => return,
        };

        state
            .space
            .outputs()
            .find(|o| {
                o.user_data().get::<UdevOutputId>()
                    == Some(&UdevOutputId {
                        device_id: surface.device_id,
                        crtc,
                    })
            })
            .cloned()
    };

    let output = match output {
        Some(o) => o,
        None => return,
    };

    let start = Instant::now();

    // 发送帧回调给所有窗口
    state.space.elements().for_each(|window| {
        window.send_frame(
            &output,
            state.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        );
    });

    // 获取渲染器并执行渲染
    let udev = state.udev_data.as_mut().expect("udev backend initialized");
    let primary_gpu = udev.primary_gpu;
    let device = match udev.backends.get_mut(&node) {
        Some(d) => d,
        None => return,
    };
    let surface = match device.surfaces.get_mut(&crtc) {
        Some(s) => s,
        None => return,
    };

    let render_node = surface.render_node.unwrap_or(primary_gpu);
    let mut renderer = match if primary_gpu == render_node {
        udev.gpus.single_renderer(&render_node)
    } else {
        let format = surface.drm_output.format();
        udev.gpus.renderer(&primary_gpu, &render_node, format)
    } {
        Ok(r) => r,
        Err(err) => {
            warn!("获取渲染器失败，跳过本帧: {:?}", err);
            return;
        }
    };

    // 使用 Space 的渲染元素来组合帧
    let space_elements = smithay::desktop::space::space_render_elements::<
        _,
        smithay::desktop::Window,
        _,
    >(&mut renderer, [&state.space], &output, 1.0)
    .unwrap_or_default();

    // 组合渲染元素：光标（z-order 最高）+ 窗口
    let mut elements: Vec<UdevRenderElement<'_>> = Vec::with_capacity(space_elements.len() + 1);

    // 光标元素：Smithay DRM compositor 会自动检测 Kind::Cursor 并分配到硬件光标平面
    if !matches!(
        state.cursor_status,
        smithay::input::pointer::CursorImageStatus::Hidden
    ) {
        if let Some(pointer) = state.seat.get_pointer() {
            let pos = pointer.current_location();
            let cursor_el = crate::cursor::cursor_square_element(pos, Scale::from(1.0));
            elements.push(UdevRenderElement::Cursor(cursor_el));
        }
    }

    elements.extend(space_elements.into_iter().map(UdevRenderElement::Space));

    // 渲染帧并提交（光标元素自动走 DRM cursor plane）
    let result = surface.drm_output.render_frame(
        &mut renderer,
        &elements,
        [0.1f32, 0.1, 0.1, 1.0],
        FrameFlags::empty(),
    );

    let reschedule = match result {
        Ok(render_result) => {
            if !render_result.is_empty {
                if let Err(err) = surface.drm_output.queue_frame(Some(())) {
                    warn!("提交帧失败: {:?}", err);
                }
                false
            } else {
                // 无 damage — 不提交帧，延迟重试
                true
            }
        }
        Err(err) => {
            warn!("渲染出错: {:#?}", err);
            true
        }
    };

    if reschedule {
        // 如果没有损坏或发生临时错误，在下一帧时间重试
        let output_refresh = match output.current_mode() {
            Some(mode) => mode.refresh,
            None => return,
        };
        let next_frame_target =
            frame_target + Duration::from_millis(1_000_000 / output_refresh as u64);
        let reschedule_timeout = Duration::from(next_frame_target).saturating_sub(Duration::ZERO);
        let timer = Timer::from_duration(reschedule_timeout.min(Duration::from_millis(16)));
        state
            .loop_handle
            .insert_source(timer, move |_, _, state| {
                render_surface(state, node, crtc, next_frame_target);
                TimeoutAction::Drop
            })
            .expect("调度帧定时器失败");
    } else {
        let elapsed = start.elapsed();
        trace!("渲染完成，耗时 {:?}", elapsed);
    }
}
