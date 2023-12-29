use std::fs::File;
use std::io::{Error, Write};
use std::net::SocketAddr;
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::ptr;

use laminar::{Packet, Socket};

use winapi::ctypes::c_void;
use winapi::um::namedpipeapi::CreatePipe;
use winapi::um::processenv::SetStdHandle;
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

#[path = "../examples/remote_terminal.rs"]
mod terminal_app;
pub use terminal_app::remote_terminal;

#[cfg(target_os = "windows")]
pub fn redirect_stdout_to_ip(ip: &'static str) -> std::io::Result<()> {
    let mut read_handle: RawHandle = ptr::null_mut();
    let mut write_handle: RawHandle = ptr::null_mut();

    unsafe {
        // Create a pipe for stdout and stderr
        if CreatePipe(&mut read_handle, &mut write_handle, ptr::null_mut(), 0) == 0 {
            let error = Error::last_os_error();
            return Err(error);
        }

        // Set stdout to our pipe handle
        if SetStdHandle(STD_OUTPUT_HANDLE, write_handle as *mut c_void) == 0 {
            let error = Error::last_os_error();
            return Err(error);
        }

        // Set stderr to our pipe handle
        if SetStdHandle(STD_ERROR_HANDLE, write_handle as *mut c_void) == 0 {
            let error = Error::last_os_error();
            return Err(error);
        }
    }

    let read_handle = ReadHandle(read_handle);

    // Spawn a thread to read from the pipe and send to a UDP destination.
    std::thread::spawn(move || {
        broadcast_pipe_at(read_handle, ip).unwrap();
    });

    Ok(())
}

fn broadcast_pipe_at(read_handle: ReadHandle, destination: &str) -> std::io::Result<()> {
    let mut pipe = unsafe { File::from_raw_handle(read_handle.0) };
    let mut broadcast = Broadcast::new(destination);

    // This runs "forever".
    std::io::copy(&mut pipe, &mut broadcast)?;
    std::mem::forget(pipe);

    Ok(())
}

struct Broadcast {
    socket: Socket,
    destination: SocketAddr,
}

impl Broadcast {
    fn new(destination: &str) -> Self {
        Self {
            socket: Socket::bind("127.0.0.1:0").unwrap(),
            destination: destination.parse().unwrap(),
        }
    }

    fn send_packet(&mut self, data: impl Into<Vec<u8>>) -> std::io::Result<()> {
        let packet = Packet::reliable_ordered(self.destination, data.into(), None);
        self.socket
            .send(packet)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        self.socket.manual_poll(std::time::Instant::now()); // important
        Ok(())
    }
}

impl Write for Broadcast {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.send_packet(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct ReadHandle(*mut c_void);
unsafe impl Send for ReadHandle {} // fight me.
