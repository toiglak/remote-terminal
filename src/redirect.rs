use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, Error};
use std::net::{SocketAddr, UdpSocket};
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::path::Path;
use std::ptr;

use winapi::ctypes::c_void;
use winapi::um::namedpipeapi::CreatePipe;
use winapi::um::processenv::{GetStdHandle, SetStdHandle};
use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

pub fn redirect_stdout_to_pipe() -> std::io::Result<()> {
    let mut read_handle: RawHandle = ptr::null_mut();
    let mut write_handle: RawHandle = ptr::null_mut();

    let _stdout_handle;

    unsafe {
        // Get the current stdout handle
        _stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);

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
        pipe_to_udp(read_handle, "127.0.0.1:7272").unwrap();
    });

    // Spawn a thread to read from the pipe and write to a file.
    // std::thread::spawn(move || {
    //     pipe_to_file(read_handle, "log.txt").unwrap();
    // });

    // let stdout_handle = ReadHandle(stdout_handle);

    // Spawn a thread to read from the pipe and write to stdout.
    // std::thread::spawn(move || {
    //     pipe_to_stdout(read_handle, stdout_handle).unwrap();
    // });

    Ok(())
}

fn pipe_to_file(read_handle: ReadHandle, file: impl AsRef<Path>) -> std::io::Result<()> {
    // Wrap the raw handle in a safe File
    let pipe = unsafe { File::from_raw_handle(read_handle.0) };

    // Create a BufReader for the file
    let reader = BufReader::new(pipe);

    // Open the output file in append mode
    let mut file = File::create(file)?;

    // UB: when `reader` is dropped `pipe` will be dropped, closing to stdout

    // Read the data from the file line by line
    for line in reader.lines() {
        // Write each line to the output file
        writeln!(file, "{}", line?)?;
    }

    Ok(())
}

fn pipe_to_udp(read_handle: ReadHandle, destination: &str) -> std::io::Result<()> {
    // Wrap the raw handle in a safe File
    let mut pipe = unsafe { File::from_raw_handle(read_handle.0) };

    let mut udp = UdpWriter {
        socket: UdpSocket::bind("127.0.0.1:0")?,
        destination: destination.parse().unwrap(),
    };

    std::io::copy(&mut pipe, &mut udp)?;

    std::mem::forget(pipe);

    Ok(())
}

fn pipe_to_stdout(read_handle: ReadHandle, stdout_handle: ReadHandle) -> std::io::Result<()> {
    // Wrap the raw handle in a safe File
    let mut stdout = unsafe { File::from_raw_handle(stdout_handle.0) };

    // Wrap the raw handle in a safe File
    let mut pipe = unsafe { File::from_raw_handle(read_handle.0) };

    std::io::copy(&mut pipe, &mut stdout)?;

    std::mem::forget(stdout);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ReadHandle(*mut c_void);
unsafe impl Send for ReadHandle {} // fight me.

//

struct UdpWriter {
    socket: UdpSocket,
    destination: SocketAddr,
}

impl Write for UdpWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.send_to(buf, &self.destination)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
