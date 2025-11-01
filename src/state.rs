pub mod client;

pub struct OneShot {
    pub socket_name: std::ffi::OsString,
    pub compositor_state: smithay::wayland::compositor::CompositorState,
    pub shm_state: smithay::wayland::shm::ShmState,
}

smithay::delegate_compositor!(OneShot);
smithay::delegate_shm!(OneShot);

impl OneShot {
    pub fn new(
        event_loop: &mut calloop::EventLoop<CalloopData>,
        display: wayland_server::Display<Self>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display_handle = display.handle();

        let compositor_state: smithay::wayland::compositor::CompositorState =
            smithay::wayland::compositor::CompositorState::new::<Self>(&display_handle);

        let shm_state: smithay::wayland::shm::ShmState =
            smithay::wayland::shm::ShmState::new::<Self>(&display_handle, vec![]);

        let socket_name: std::ffi::OsString = Self::init_wayland_listener(display, event_loop);

        Ok(Self {
            socket_name,
            compositor_state,
            shm_state,
        })
    }

    fn init_wayland_listener(
        display: wayland_server::Display<Self>,
        event_loop: &mut calloop::EventLoop<CalloopData>,
    ) -> std::ffi::OsString {
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
                    .insert_client(client_stream, std::sync::Arc::new(self::client::ClientState::default()))
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

impl smithay::wayland::compositor::CompositorHandler for OneShot {
    fn compositor_state(&mut self) -> &mut smithay::wayland::compositor::CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(
        &self,
        client: &'a wayland_server::Client,
    ) -> &'a smithay::wayland::compositor::CompositorClientState {
        &client.get_data::<self::client::ClientState>().unwrap().compositor_state
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

pub struct CalloopData {
    pub state: OneShot,
    pub display_handle: wayland_server::DisplayHandle,
}
