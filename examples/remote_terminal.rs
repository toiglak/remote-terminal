use std::io::BufReader;

use laminar::{Socket, SocketEvent};

/// Use this function inside of your binary, for example `cargo run --bin remote_terminal`.
pub fn remote_terminal(ip: &'static str) -> std::io::Result<()> {
    let mut socket = Socket::bind(ip).unwrap();
    let (_, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
    let _thread = std::thread::spawn(move || socket.start_polling());

    loop {
        if let Ok(SocketEvent::Packet(packet)) = receiver.recv() {
            let msg = packet.payload();
            std::io::copy(&mut BufReader::new(msg), &mut std::io::stdout())?;
        }
    }
}

pub const BROADCAST_ADDRESS: &str = "127.0.0.1:7227";

#[allow(unused)]
fn main() -> std::io::Result<()> {
    remote_terminal(BROADCAST_ADDRESS)
}
