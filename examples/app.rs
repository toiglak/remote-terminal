use remote_terminal::redirect_stdout_to_ip;

pub const BROADCAST_AT: &str = "127.0.0.1:7227";

fn main() -> std::io::Result<()> {
    redirect_stdout_to_ip(BROADCAST_AT)?;

    std::env::set_var("RUST_LOG", "info");
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
