use calloop::EventLoop;
use std::{ffi::OsString, sync::Arc};
use wayland_server::{Display, DisplayHandle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Event loop
    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;
    let event_loop_handle = event_loop.handle();

    // Display
    let display: Display<OneShot> = Display::new()?;
    let display_handle: wayland_server::DisplayHandle = display.handle();

    // Oneshot State
    let state: OneShot = OneShot::new(&mut event_loop, display)?;

    let mut data: CalloopData = CalloopData {
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

struct OneShot {
    pub socket_name: OsString,
    pub compositor_state: smithay::wayland::compositor::CompositorState,
    pub shm_state: smithay::wayland::shm::ShmState,
}

impl OneShot {
    fn new(
        event_loop: &mut calloop::EventLoop<CalloopData>,
        display: Display<Self>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display_handle = display.handle();

        let compositor_state: smithay::wayland::compositor::CompositorState =
            smithay::wayland::compositor::CompositorState::new::<Self>(&display_handle);

        let shm_state: smithay::wayland::shm::ShmState =
            smithay::wayland::shm::ShmState::new::<Self>(&display_handle, vec![]);

        let socket_name: OsString = Self::init_wayland_listener(display, event_loop);

        Ok(Self {
            socket_name,
            compositor_state,
            shm_state,
        })
    }

    fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<CalloopData>,
    ) -> OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = smithay::wayland::socket::ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = listening_socket.socket_name().to_os_string();

        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                // Inside the callback, you should insert the client into the display.
                //
                // You may also associate some data with the client when inserting the client.
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("Failed to init the wayland event source.");

        // You also need to add the display itself to the event loop, so that client events will be processed by wayland-server.
        loop_handle
            .insert_source(
                calloop::generic::Generic::new(
                    display,
                    calloop::Interest::READ,
                    calloop::Mode::Level,
                ),
                |_, display, state| {
                    // Safety: we don't drop the display
                    unsafe {
                        display
                            .get_mut()
                            .dispatch_clients(&mut state.state)
                            .unwrap();
                    }
                    Ok(calloop::PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }
}

struct CalloopData {
    state: OneShot,
    display_handle: DisplayHandle,
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: smithay::wayland::compositor::CompositorClientState,
}

smithay::delegate_compositor!(OneShot);
smithay::delegate_shm!(OneShot);

impl wayland_server::backend::ClientData for ClientState {
    fn initialized(&self, _client_id: wayland_server::backend::ClientId) {}
    fn disconnected(
        &self,
        _client_id: wayland_server::backend::ClientId,
        _reason: wayland_server::backend::DisconnectReason,
    ) {
    }
}

impl smithay::wayland::compositor::CompositorHandler for OneShot {
    fn compositor_state(&mut self) -> &mut smithay::wayland::compositor::CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a wayland_server::Client,
    ) -> &'a smithay::wayland::compositor::CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, _surface: &wayland_server::protocol::wl_surface::WlSurface) {}
}

impl smithay::wayland::shm::ShmHandler for OneShot {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.shm_state
    }
}

impl smithay::wayland::buffer::BufferHandler for OneShot {
    fn buffer_destroyed(&mut self, _buffer: &wayland_server::protocol::wl_buffer::WlBuffer) {}
}
