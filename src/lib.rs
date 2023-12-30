use std::fs::File;
use std::io::{Error, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::ptr;

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
    let mut broadcast = Broadcast::new(destination)?;

    // This runs "forever".
    std::io::copy(&mut pipe, &mut broadcast)?;
    std::mem::forget(pipe);

    Ok(())
}

struct Broadcast {
    destination: SocketAddr,
    stream: Option<TcpStream>,
}

impl Broadcast {
    fn new(destination: impl ToSocketAddrs) -> std::io::Result<Self> {
        let destination = destination.to_socket_addrs()?.next().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No valid address provided",
            )
        })?;
        Ok(Self {
            destination,
            stream: None,
        })
    }

    fn connect_if_none(&mut self) {
        if self.stream.is_none() {
            if let Ok(stream) = TcpStream::connect(self.destination) {
                self.stream = Some(stream);
            }
        }
    }
}

impl Write for Broadcast {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.connect_if_none();

        if let Some(stream) = self.stream.as_mut() {
            if let Err(_) = stream.write_all(buf) {
                self.stream = None;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            if let Err(_) = stream.flush() {
                self.stream = None;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct ReadHandle(*mut c_void);
unsafe impl Send for ReadHandle {} // fight me.
