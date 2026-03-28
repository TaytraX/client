use std::sync::Mutex;
use std::io::Read;
use std::net::{TcpStream, UdpSocket};
use bytemuck::Pod;

pub struct Connection {
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
}
#[derive(Copy, Clone, Pod, bytemuck::Zeroable)]
#[repr(C)]
struct TransformationComponent {
    position: [f64; 3],
    rotation: [f64; 3],
}

static COMPONENTS: Mutex<Vec<TransformationComponent>> = Mutex::new(Vec::new());
pub static CHUNKS: Mutex<Vec<[f64; 32768]>> = Mutex::new(Vec::new());
pub static mut CHUNKS_DIRTY: bool = false;

impl Connection {
    pub fn new() -> Self {
        let tcp_stream = TcpStream::connect("127.0.0.1:5000").unwrap();
        println!("TCP connection established");

        let udp_socket = UdpSocket::bind("127.0.0.1:9001").unwrap();
        udp_socket.set_nonblocking(true).unwrap();
        println!("connected to udp");

        Self {
            tcp_stream,
            udp_socket,
        }
    }

    pub fn update(&mut self) {
        self.receive_chunk();
        self.receive_component();
    }

    fn receive_chunk(&mut self) {
        let mut buf = [0; 262144];

        match self.tcp_stream.read_exact(&mut buf) {
            Ok(_) => {
                println!("received chunk");
                unsafe { CHUNKS_DIRTY = true; }
                let teste = unsafe { std::mem::transmute(buf) };
                CHUNKS.try_lock().unwrap().push(teste);
            }
            Err(_) => { return; }
        }
    }

    fn receive_component(&mut self) {
        let mut buf = [0; 65507];
        match self.udp_socket.recv_from(&mut buf) {
            Ok((_size, _)) => {
                let len = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
                COMPONENTS.try_lock().unwrap().extend_from_slice(bytemuck::cast_slice(&buf[4..4 + len * size_of::<TransformationComponent>()]))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("UDP error: {e}"),
        }
    }
}