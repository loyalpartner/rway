// handlers/screencopy.rs — wlr-screencopy-unstable-v1 protocol implementation
//
// Enables grim and other screencopy clients to capture output frames.
// Reference: wlroots types/wlr_screencopy_v1.c

use smithay::{
    output::Output,
    reexports::wayland_server::{
        protocol::{wl_buffer::WlBuffer, wl_output::WlOutput, wl_shm},
        Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource,
    },
    wayland::shm,
};
use wayland_protocols_wlr::screencopy::v1::server::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::{self, ZwlrScreencopyManagerV1},
};

use crate::state::RwayState;

const SCREENCOPY_VERSION: u32 = 3;
const BPP: u32 = 4; // Argb8888 = 4 bytes per pixel

// ── Data types ──

/// User data attached to each ZwlrScreencopyFrameV1 resource.
pub(crate) struct ScreencopyFrameData {
    pub output: Output,
    pub src_x: u32,
    pub src_y: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

/// A pending screencopy request waiting for the next render.
pub(crate) struct PendingScreencopy {
    pub frame: ZwlrScreencopyFrameV1,
    pub buffer: WlBuffer,
    pub output: Output,
    pub src_x: u32,
    pub src_y: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub with_damage: bool,
}

// ── GlobalDispatch: client binds to zwlr_screencopy_manager_v1 ──

impl GlobalDispatch<ZwlrScreencopyManagerV1, ()> for RwayState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrScreencopyManagerV1>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, ());
    }
}

// ── Dispatch: manager requests ──

impl Dispatch<ZwlrScreencopyManagerV1, ()> for RwayState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &ZwlrScreencopyManagerV1,
        request: zwlr_screencopy_manager_v1::Request,
        _data: &(),
        _dh: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            zwlr_screencopy_manager_v1::Request::CaptureOutput {
                frame,
                overlay_cursor: _,
                output,
            } => {
                handle_capture_output(frame, &output, None, data_init);
            }
            zwlr_screencopy_manager_v1::Request::CaptureOutputRegion {
                frame,
                overlay_cursor: _,
                output,
                x,
                y,
                width,
                height,
            } => {
                let region = Some((x, y, width, height));
                handle_capture_output(frame, &output, region, data_init);
            }
            zwlr_screencopy_manager_v1::Request::Destroy => {}
            _ => {}
        }
    }
}

fn handle_capture_output(
    frame: New<ZwlrScreencopyFrameV1>,
    wl_output: &WlOutput,
    region: Option<(i32, i32, i32, i32)>,
    data_init: &mut DataInit<'_, RwayState>,
) {
    let Some(output) = Output::from_resource(wl_output) else {
        let frame_data = ScreencopyFrameData {
            output: Output::new(
                "dummy".to_string(),
                smithay::output::PhysicalProperties {
                    size: (0, 0).into(),
                    subpixel: smithay::output::Subpixel::Unknown,
                    make: String::new(),
                    model: String::new(),
                },
            ),
            src_x: 0,
            src_y: 0,
            width: 0,
            height: 0,
            stride: 0,
        };
        let frame = data_init.init(frame, frame_data);
        frame.failed();
        return;
    };

    let Some(mode) = output.current_mode() else {
        let frame_data = ScreencopyFrameData {
            output: output.clone(),
            src_x: 0,
            src_y: 0,
            width: 0,
            height: 0,
            stride: 0,
        };
        let frame = data_init.init(frame, frame_data);
        frame.failed();
        return;
    };

    let (src_x, src_y, width, height) = if let Some((x, y, w, h)) = region {
        let out_w = mode.size.w as u32;
        let out_h = mode.size.h as u32;
        // Reject negative or out-of-bounds region
        if x < 0
            || y < 0
            || w <= 0
            || h <= 0
            || x as u32 + w as u32 > out_w
            || y as u32 + h as u32 > out_h
        {
            let frame_data = ScreencopyFrameData {
                output: output.clone(),
                src_x: 0,
                src_y: 0,
                width: 0,
                height: 0,
                stride: 0,
            };
            let frame = data_init.init(frame, frame_data);
            frame.failed();
            return;
        }
        (x as u32, y as u32, w as u32, h as u32)
    } else {
        (0, 0, mode.size.w as u32, mode.size.h as u32)
    };
    let stride = width * BPP;

    let frame_data = ScreencopyFrameData {
        output: output.clone(),
        src_x,
        src_y,
        width,
        height,
        stride,
    };

    let frame = data_init.init(frame, frame_data);

    // Tell client what SHM format to use
    // Advertise Argb8888 (BGRA) which grim requires.
    // We read as Abgr8888 (RGBA) from GL and swap R↔B when copying to the SHM buffer.
    frame.buffer(wl_shm::Format::Argb8888, width, height, stride);

    // Version 3: signal that all buffer types have been enumerated
    if frame.version() >= 3 {
        frame.buffer_done();
    }
}

// ── Dispatch: frame requests ──

impl Dispatch<ZwlrScreencopyFrameV1, ScreencopyFrameData> for RwayState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &ZwlrScreencopyFrameV1,
        request: zwlr_screencopy_frame_v1::Request,
        data: &ScreencopyFrameData,
        _dh: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        let with_damage = matches!(
            request,
            zwlr_screencopy_frame_v1::Request::CopyWithDamage { .. }
        );
        match request {
            zwlr_screencopy_frame_v1::Request::Copy { buffer }
            | zwlr_screencopy_frame_v1::Request::CopyWithDamage { buffer } => {
                // Reject duplicate copy on same frame
                if state
                    .pending_screencopies
                    .iter()
                    .any(|p| p.frame == *resource)
                {
                    resource.post_error(
                        zwlr_screencopy_frame_v1::Error::AlreadyUsed,
                        "copy already requested for this frame",
                    );
                    return;
                }

                // Validate SHM buffer
                let valid = shm::with_buffer_contents(&buffer, |_ptr, _len, buf_data| {
                    buf_data.format == wl_shm::Format::Argb8888
                        && buf_data.width == data.width as i32
                        && buf_data.height == data.height as i32
                        && buf_data.stride == data.stride as i32
                });
                match valid {
                    Ok(true) => {}
                    _ => {
                        resource.post_error(
                            zwlr_screencopy_frame_v1::Error::InvalidBuffer,
                            "invalid buffer dimensions, format, or not SHM",
                        );
                        return;
                    }
                }

                state.pending_screencopies.push(PendingScreencopy {
                    frame: resource.clone(),
                    buffer,
                    output: data.output.clone(),
                    src_x: data.src_x,
                    src_y: data.src_y,
                    width: data.width,
                    height: data.height,
                    stride: data.stride,
                    with_damage,
                });

                state.schedule_redraw();
            }
            zwlr_screencopy_frame_v1::Request::Destroy => {}
            _ => {}
        }
    }

    fn destroyed(
        state: &mut Self,
        _client: wayland_server::backend::ClientId,
        resource: &ZwlrScreencopyFrameV1,
        _data: &ScreencopyFrameData,
    ) {
        state.pending_screencopies.retain(|p| p.frame != *resource);
    }
}

// ── Screencopy fulfillment (called from render loops) ──

/// Captured framebuffer pixels, stored between readback and SHM copy.
pub(crate) struct CapturedFrame {
    pub data: Vec<u8>,
    pub stride: usize,
    pub height: usize,
    pub flipped: bool,
}

/// Copy captured pixels into pending SHM buffers and send ready events.
///
/// Called after submit(), when EGL state is safe.
pub(crate) fn fulfill_screencopy(state: &mut RwayState, captured: &CapturedFrame, output: &Output) {
    let elapsed = state.start_time.elapsed();

    let mut i = 0;
    while i < state.pending_screencopies.len() {
        if state.pending_screencopies[i].output == *output {
            let req = state.pending_screencopies.remove(i);
            copy_to_shm_and_signal(
                &req,
                &captured.data,
                captured.stride,
                captured.height,
                captured.flipped,
                &elapsed,
            );
        } else {
            i += 1;
        }
    }
}

/// Copy pixel data from GPU readback into the client's SHM buffer and send protocol events.
///
/// Supports both full-output and sub-region capture. `src_data` is always the full
/// framebuffer; `req.src_x/src_y` specify the top-left corner of the region to extract.
fn copy_to_shm_and_signal(
    req: &PendingScreencopy,
    src_data: &[u8],
    src_stride: usize,
    src_height: usize,
    flipped: bool,
    elapsed: &std::time::Duration,
) {
    let result = shm::with_buffer_contents_mut(&req.buffer, |ptr, len, _buf_data| {
        let dst_stride = req.stride as usize;
        let region_h = req.height as usize;
        let region_w = req.width as usize;
        let src_x_bytes = req.src_x as usize * BPP as usize;

        // SAFETY: SHM is shared memory — the client may write concurrently (inherent
        // wl_shm limitation). We accept this risk as all wl_shm compositors do.
        // copy_nonoverlapping is used for each pixel to minimize the race window.
        unsafe {
            let dst = std::slice::from_raw_parts_mut(ptr, len);
            for y in 0..region_h {
                let abs_y = req.src_y as usize + y;
                let src_row = if flipped {
                    src_height - 1 - abs_y
                } else {
                    abs_y
                };
                let src_row_start = src_row * src_stride + src_x_bytes;
                let dst_row_start = y * dst_stride;

                // Copy pixel-by-pixel with RGBA→BGRA channel swap (R↔B)
                // GL reads as Abgr8888 (R,G,B,A bytes), grim expects Argb8888 (B,G,R,A bytes)
                for x in 0..region_w {
                    let si = src_row_start + x * 4;
                    let di = dst_row_start + x * 4;
                    if si + 4 <= src_data.len() && di + 4 <= dst.len() {
                        dst[di] = src_data[si + 2]; // B ← src[2]
                        dst[di + 1] = src_data[si + 1]; // G ← src[1]
                        dst[di + 2] = src_data[si]; // R ← src[0]
                        dst[di + 3] = src_data[si + 3]; // A ← src[3]
                    }
                }
            }
        }
    });

    if result.is_err() {
        tracing::warn!("screencopy: failed to write to SHM buffer");
        req.frame.failed();
        return;
    }

    // No y_invert — we already flipped during copy
    req.frame.flags(zwlr_screencopy_frame_v1::Flags::empty());

    // copy_with_damage requires a damage event before ready
    if req.with_damage {
        req.frame.damage(0, 0, req.width, req.height);
    }

    let secs = elapsed.as_secs();
    let tv_sec_hi = (secs >> 32) as u32;
    let tv_sec_lo = secs as u32;
    let tv_nsec = elapsed.subsec_nanos();
    req.frame.ready(tv_sec_hi, tv_sec_lo, tv_nsec);
}

/// Send `failed` to all pending requests for a given output.
pub(crate) fn fail_all_for_output(pending: &mut Vec<PendingScreencopy>, output: &Output) {
    let mut i = 0;
    while i < pending.len() {
        if pending[i].output == *output {
            let req = pending.remove(i);
            req.frame.failed();
        } else {
            i += 1;
        }
    }
}

/// Register the screencopy global.
pub(crate) fn init_screencopy(dh: &DisplayHandle) {
    dh.create_global::<RwayState, ZwlrScreencopyManagerV1, ()>(SCREENCOPY_VERSION, ());
}

// Re-export wayland_server for the `destroyed` callback ClientId type
use smithay::reexports::wayland_server;
