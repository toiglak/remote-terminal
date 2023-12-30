use std::net::TcpListener;

/// Use this function inside of your binary, for example `cargo run --bin remote_terminal`.
pub fn remote_terminal(ip: &'static str) -> std::io::Result<()> {
    let stream = TcpListener::bind(ip).unwrap();

    loop {
        let (mut socket, _) = stream.accept()?;
        let _ = std::io::copy(&mut socket, &mut std::io::stdout());
    }
}

pub const BROADCAST_ADDRESS: &str = "127.0.0.1:7227";

#[allow(unused)]
fn main() -> std::io::Result<()> {
    remote_terminal(BROADCAST_ADDRESS)
}
