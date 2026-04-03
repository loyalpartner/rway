// backend/winit.rs — Winit backend (development/testing mode)

use std::time::Duration;

use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::EventLoop,
    utils::{Rectangle, Scale, Transform},
};

use crate::{border::BorderConfig, render, state::RwayState};

/// Initialize the Winit backend: create window, output, damage tracker, and register with event loop
pub(crate) fn init_winit(
    event_loop: &mut EventLoop<RwayState>,
    state: &mut RwayState,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut backend, winit) = winit::init::<GlesRenderer>()?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

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

    state.space.map_output(&output, (0, 0));

    let win_size = backend.window_size();
    state.init_tiling_output(win_size.w, win_size.h);

    let mut damage_tracker = OutputDamageTracker::from_output(&output);
    let border_config = BorderConfig::from_config(&state.config);

    event_loop
        .handle()
        .insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::Resized { size, .. } => {
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
                    state.process_input_event(event);
                    backend.window().request_redraw();
                }

                WinitEvent::Redraw => {
                    let _has_animations = state.update_animations();
                    state.needs_redraw = false;

                    let size = backend.window_size();
                    let damage = Rectangle::from_size(size);

                    {
                        let scale = Scale::from(1.0_f64);

                        // Shared pipeline: generate overlay elements (borders + cursor)
                        let custom_elements =
                            render::overlay_elements(state, scale, &border_config);

                        let Ok((renderer, mut framebuffer)) = backend.bind() else {
                            tracing::warn!("Failed to bind winit backend framebuffer");
                            return;
                        };

                        // space::render_output handles Space elements internally,
                        // we only provide overlays as custom_elements
                        if let Err(e) = smithay::desktop::space::render_output::<
                            _,
                            smithay::backend::renderer::element::solid::SolidColorRenderElement,
                            _,
                            _,
                        >(
                            &output,
                            renderer,
                            &mut framebuffer,
                            1.0,
                            0,
                            [&state.space],
                            &custom_elements,
                            &mut damage_tracker,
                            render::CLEAR_COLOR,
                        ) {
                            tracing::warn!("Failed to render output: {:?}", e);
                        }
                    }

                    // submit() provides VSync throttling (~60fps)
                    if let Err(e) = backend.submit(Some(&[damage])) {
                        tracing::warn!("Failed to submit winit frame: {:?}", e);
                    }

                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        )
                    });

                    state.space.refresh();
                    state.popups.cleanup();
                    state.cleanup_dead_windows();

                    let _ = state.display_handle.flush_clients();

                    // Always request next frame. VSync via submit() throttles
                    // to ~60fps. Damage tracking skips GPU work when unchanged.
                    backend.window().request_redraw();
                }

                WinitEvent::CloseRequested => {
                    state.loop_signal.stop();
                }

                _ => (),
            }
        })?;

    Ok(())
}
