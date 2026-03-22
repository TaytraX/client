mod app;
mod networking;
mod chunky;

use std::sync::mpsc;
use networking::*;
use winit::event_loop::{ControlFlow, EventLoop};
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut app = app::App::new(rx);
    let event_loop = EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    thread::spawn(move || {
        loop {
            tx.send(get_chunk()).unwrap()
        }
    });
    let _ = event_loop.run_app(&mut app);
}