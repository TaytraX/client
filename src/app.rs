use std::sync::{mpsc, Arc};
use instant::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};
use renderer::render_backend::context::Context;
use renderer::render_backend::State;
use renderer::render_backend::vertex::Vertex;

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
    last_time: Instant,
    rx: mpsc::Receiver<Vec<Vertex>>,
}

impl App {
    pub fn new(receiver: mpsc::Receiver<Vec<Vertex>>) -> Self {
        Self {
            window: None,
            state: None,
            last_time: Instant::now(),
            rx: receiver,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attribut = WindowAttributes::default();
        window_attribut.title = "Wgpu Learning Programm".to_string();
        window_attribut.inner_size = Some(Size::Physical(PhysicalSize::new(
            1200,
            800
        )));
        let window = Arc::new(event_loop.create_window(window_attribut).unwrap());
        let context = pollster::block_on(Context::new(&window));

        self.state = Some(pollster::block_on(State::new(context)));

        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return
        };

        let window = match &mut self.window {
            Some(window) => window,
            None => return
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let dt = self.last_time.elapsed();
                self.last_time = Instant::now();

                if let Ok(vertices) = self.rx.try_recv() {
                    state.update_vert(vertices);
                }

                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }

                }
            },
            WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {
                    state.camera_controller.handle_key(code, key_state.is_pressed());
                },
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let state = if let Some(state) = &mut self.state {
            state
        } else {
            return;
        };

        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                state
                    .camera_controller
                    .handle_mouse(dx, dy);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}