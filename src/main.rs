use redirect::redirect_stdout_to_pipe;

mod redirect;

fn main() -> std::io::Result<()> {
    redirect_stdout_to_pipe()?;

    env_logger::init();

    for thread in 0..10 {
        std::thread::spawn(move || {
            for i in 0..10 {
                log::info!("Thread {} says {}!", thread, i);
            }
        });
    }

    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}
