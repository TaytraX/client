use winit::event_loop::ControlFlow;

mod app;

fn main() {
    let mut app = app::App::new();
    let event_loop = winit::event_loop::EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);
    let _ = event_loop.run_app(&mut app);
}