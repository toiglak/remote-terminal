fn _redirect_stdout_to_file() -> Result<(), Error> {
    let file = File::create("C:/Users/idzik/Desktop/log.txt")?;

    unsafe {
        let h_file = file.as_raw_handle() as *mut winapi::ctypes::c_void;

        if SetStdHandle(STD_OUTPUT_HANDLE, h_file) == 0 {
            let error = Error::last_os_error();
            return Err(error);
        }
    }

    std::mem::forget(file);

    Ok(())
}

fn _redirect_and_undo() -> Result<(), Error> {
    let file = File::create("C:/Users/idzik/Desktop/log.txt")?;

    unsafe {
        let h_file = file.as_raw_handle() as *mut winapi::ctypes::c_void;
        let h_stdout = GetStdHandle(STD_OUTPUT_HANDLE);

        if SetStdHandle(STD_OUTPUT_HANDLE, h_file) == 0 {
            let error = Error::last_os_error();
            CloseHandle(h_file);
            return Err(error);
        }

        println!("This will be redirected to the file.");

        log::info!("So cool!");

        if SetStdHandle(STD_OUTPUT_HANDLE, h_stdout) == 0 {
            let error = Error::last_os_error();
            CloseHandle(h_file);
            return Err(error);
        }

        if CloseHandle(h_file) == 0 {
            return Err(Error::last_os_error());
        }

        log::info!("So cool!");

        println!("Done!");
    }

    Ok(())
}
