// backend/winit.rs — Winit backend (development/testing mode)
//
// Follows cosmic-comp's architecture exactly:
// - Backend state stored in RwayState
// - Two calloop::ping sources: event_ping pumps winit events,
//   render_ping triggers rendering
// - The cycle: event_ping → dispatch → ping(render) → render+submit(VSync) → ping(event)
// - submit() provides VSync throttling (~60fps), idle CPU near zero

use std::time::Duration;

use smithay::{
    backend::{
        renderer::{damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::calloop::{self, EventLoop},
    utils::{Rectangle, Scale, Transform},
};

use crate::{border::BorderConfig, render, state::RwayState};

/// Winit backend state, stored in RwayState.
pub(crate) struct WinitState {
    pub backend: smithay::backend::winit::WinitGraphicsBackend<GlesRenderer>,
    pub output: Output,
    damage_tracker: OutputDamageTracker,
    border_config: BorderConfig,
}

impl WinitState {
    /// Render one frame. Returns true if animations are still running.
    pub fn render_frame(&mut self, state: &mut RwayState) -> bool {
        let has_animations = state.update_animations();
        state.needs_redraw = false;

        let size = self.backend.window_size();
        let damage = Rectangle::from_size(size);

        {
            let scale = Scale::from(1.0_f64);
            let custom_elements = render::overlay_elements(state, scale, &self.border_config);

            let Ok((renderer, mut framebuffer)) = self.backend.bind() else {
                tracing::warn!("Failed to bind winit backend framebuffer");
                return has_animations;
            };

            if let Err(e) = smithay::desktop::space::render_output::<
                _,
                smithay::backend::renderer::element::solid::SolidColorRenderElement,
                _,
                _,
            >(
                &self.output,
                renderer,
                &mut framebuffer,
                1.0,
                0,
                [&state.space],
                &custom_elements,
                &mut self.damage_tracker,
                render::CLEAR_COLOR,
            ) {
                tracing::warn!("Failed to render output: {:?}", e);
            }
        }

        // submit() blocks for VSync — this is what throttles the loop to ~60fps
        if let Err(e) = self.backend.submit(Some(&[damage])) {
            tracing::warn!("Failed to submit winit frame: {:?}", e);
        }

        let output = &self.output;
        state.space.elements().for_each(|window| {
            window.send_frame(
                output,
                state.start_time.elapsed(),
                Some(Duration::ZERO),
                |_, _| Some(output.clone()),
            )
        });

        state.space.refresh();
        state.popups.cleanup();
        state.cleanup_dead_windows();

        let _ = state.display_handle.flush_clients();

        has_animations
    }
}

/// Initialize the Winit backend (cosmic-comp pattern).
pub(crate) fn init_winit(
    event_loop: &mut EventLoop<RwayState>,
    state: &mut RwayState,
) -> Result<(), Box<dyn std::error::Error>> {
    let (backend, mut input) = winit::init::<GlesRenderer>()?;

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

    let damage_tracker = OutputDamageTracker::from_output(&output);
    let border_config = BorderConfig::from_config(&state.config);

    state.winit = Some(WinitState {
        backend,
        output,
        damage_tracker,
        border_config,
    });

    // Two pings forming a VSync-throttled cycle (cosmic-comp pattern):
    //   event_ping → dispatch winit events → ping(render)
    //   render_ping → render + submit(VSync wait) → ping(event)
    let (event_ping, event_source) = calloop::ping::make_ping()?;
    let (render_ping, render_source) = calloop::ping::make_ping()?;

    state.render_ping = Some(render_ping.clone());

    // Render handler: render frame, then re-ping event to continue the cycle
    let event_ping_from_render = event_ping.clone();
    let render_ping_retry = render_ping.clone();
    event_loop
        .handle()
        .insert_source(render_source, move |_, _, state| {
            let Some(mut winit) = state.winit.take() else {
                return;
            };
            let has_animations = winit.render_frame(state);
            state.winit = Some(winit);

            // Continue the cycle: after VSync-throttled render, pump events again
            event_ping_from_render.ping();

            // If animations still running or new content, render again next cycle
            if has_animations || state.needs_redraw {
                render_ping_retry.ping();
            }
        })?;

    // Event handler: pump winit events, then trigger render
    let event_ping_loop = event_ping.clone();
    let render_ping_event = render_ping.clone();
    event_loop
        .handle()
        .insert_source(event_source, move |_, _, state| {
            use smithay::reexports::winit::platform::pump_events::PumpStatus;
            match input
                .dispatch_new_events(|event| state.process_winit_event(event, &render_ping_event))
            {
                PumpStatus::Continue => {
                    // Re-ping both to keep the cycle alive
                    event_ping_loop.ping();
                    render_ping_event.ping();
                }
                PumpStatus::Exit(_) => {
                    state.loop_signal.stop();
                }
            }
        })?;

    // Kick off the cycle
    event_ping.ping();

    Ok(())
}

impl RwayState {
    pub fn process_winit_event(&mut self, event: WinitEvent, render_ping: &calloop::ping::Ping) {
        match event {
            WinitEvent::Resized { size, .. } => {
                if let Some(winit) = &self.winit {
                    winit.output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                }
                render_ping.ping();
            }
            WinitEvent::Redraw => {
                render_ping.ping();
            }
            WinitEvent::Input(event) => {
                self.process_input_event(event);
                render_ping.ping();
            }
            WinitEvent::CloseRequested => {
                self.loop_signal.stop();
            }
            _ => {}
        }
    }
}
