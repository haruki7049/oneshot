fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Event loop
    let mut event_loop: calloop::EventLoop<oneshot::state::CalloopData> = calloop::EventLoop::try_new()?;
    let event_loop_handle = event_loop.handle();

    // Display
    let display: wayland_server::Display<oneshot::state::OneShot> = wayland_server::Display::new()?;
    let display_handle: wayland_server::DisplayHandle = display.handle();

    // Oneshot State
    let state: oneshot::state::OneShot = oneshot::state::OneShot::new(&mut event_loop, display)?;

    let mut data: oneshot::state::CalloopData = oneshot::state::CalloopData {
        state,
        display_handle,
    };

    println!(
        "Wayland compositor OneShot listening on {} ...",
        data.state.socket_name.to_string_lossy()
    );

    let timer_handle = event_loop_handle.clone();
    let timer = calloop::timer::Timer::from_duration(std::time::Duration::from_secs(3));

    timer_handle.insert_source(timer, move |_, _, _| {
        println!("Shutdown by panicking...");
        panic!("SHUTDOWN");
    })?;

    println!("Starting TTY session...");

    event_loop.run(None, &mut data, move |data| {
        data.display_handle.flush_clients().unwrap();
    })?;

    Ok(())
}
