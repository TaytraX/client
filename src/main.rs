mod app;
mod networking;
mod chunky;
mod player;

use std::sync::mpsc;
use networking::*;
use winit::event_loop::{ControlFlow, EventLoop};
use std::thread;
use crate::chunky::build_chunk;

fn main() {
    let (tx, rx) = mpsc::channel();
    let mut app = app::App::new(rx);
    let event_loop = EventLoop::with_user_event().build().unwrap();
    
    event_loop.set_control_flow(ControlFlow::Poll);

    thread::spawn(move || {
        let mut connection: Connection = Connection::new();
        loop {
            connection.update();
        
            unsafe {
                if CHUNKS_DIRTY {
                    let vertices = build_chunk();
                    tx.send(vertices).unwrap();
                    println!("Sending chunk");
                }
            }
        }
    });
    let _ = event_loop.run_app(&mut app);
}