use smithay::{
    backend::renderer::gles::GlesRenderer, backend::winit, desktop::PopupManager, desktop::Space,
    input::keyboard::Keysym, input::keyboard::LedState, input::pointer::CursorImageStatus,
    input::pointer::PointerHandle, input::Seat, input::SeatState, output::Mode, output::Output,
    output::PhysicalProperties, output::Subpixel, reexports::calloop::EventLoop,
    reexports::wayland_server::protocol::wl_surface::WlSurface, reexports::wayland_server::Display,
    reexports::wayland_server::DisplayHandle, utils::Clock, utils::Logical, utils::Monotonic,
    wayland::commit_timing::CommitTimingManagerState, wayland::compositor::CompositorState,
    wayland::fifo::FifoManagerState, wayland::fractional_scale::FractionalScaleState,
    wayland::keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitState,
    wayland::output::OutputManagerState, wayland::presentation::PresentationState,
    wayland::selection::data_device::DataDeviceState,
    wayland::selection::primary_selection::PrimarySelectionState,
    wayland::selection::wlr_data_control::DataControlState,
    wayland::shell::wlr_layer::WlrLayerShellState,
    wayland::shell::xdg::decoration::XdgDecorationState, wayland::shell::xdg::XdgShellState,
    wayland::shm::ShmState, wayland::single_pixel_buffer::SinglePixelBufferState,
    wayland::viewporter::ViewporterState, wayland::xdg_activation::XdgActivationState,
    wayland::xdg_foreign::XdgForeignState,
};
use std::sync::{atomic::AtomicBool, Arc};
use tracing::error;

const OUTPUT_NAME: &str = "winit";

#[derive(Debug)]
pub struct DndIcon {
    pub surface: WlSurface,
    pub offset: Point<i32, Logical>,
}

struct OneshotState<BackendData: Backend + 'static> {
    pub backend_data: BackendData,
    pub socket_name: Option<String>,
    pub display_handle: DisplayHandle,
    pub running: Arc<AtomicBool>,
    pub handle: LoopHandle<'static, OneshotState<BackendData>>,

    // desktop
    pub space: Space<WindowElement>,
    pub popups: PopupManager,

    // smithay state
    pub compositor_state: CompositorState,
    pub data_device_state: DataDeviceState,
    pub layer_shell_state: WlrLayerShellState,
    pub output_manager_state: OutputManagerState,
    pub primary_selection_state: PrimarySelectionState,
    pub data_control_state: DataControlState,
    pub seat_state: SeatState<OneshotState<BackendData>>,
    pub keyboard_shortcuts_inhibit_state: KeyboardShortcutsInhibitState,
    pub shm_state: ShmState,
    pub viewporter_state: ViewporterState,
    pub xdg_activation_state: XdgActivationState,
    pub xdg_decoration_state: XdgDecorationState,
    pub xdg_shell_state: XdgShellState,
    pub presentation_state: PresentationState,
    pub fractional_scale_manager_state: FractionalScaleManagerState,
    pub xdg_foreign_state: XdgForeignState,
    pub single_pixel_buffer_state: SinglePixelBufferState,
    pub fifo_manager_state: FifoManagerState,
    pub commit_timing_manager_state: CommitTimingManagerState,

    pub dnd_icon: Option<DndIcon>,

    // input-related fields
    pub suppressed_keys: Vec<Keysym>,
    pub cursor_status: CursorImageStatus,
    pub seat_name: String,
    pub seat: Seat<OneshotState<BackendData>>,
    pub clock: Clock<Monotonic>,
    pub pointer: PointerHandle<OneshotState<BackendData>>,

    pub show_window_preview: bool,
}

impl<BackendData: Backend> SeatHander for OneshotState<BackendData> {
    type KeyboardFocus = KeyboardFocusTarget;
}

trait Backend {
    const HAS_RELATIVE_MOTION: bool = false;
    const HAS_GESTURES: bool = false;
    fn seat_name(&self) -> String;
    fn reset_buffers(&mut self, output: &Output);
    fn early_import(&mut self, surface: &WlSurface);
    fn update_led_state(&mut self, led_state: LedState);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run()?;

    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // let mut event_loop = EventLoop::try_new()?;
    let display: Display<()> = Display::new()?;
    let mut display_handle = display.handle();

    let (backend, winit) = match winit::init::<GlesRenderer>() {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to initialize Winit backend: {}", err);
            return Ok(());
        }
    };
    let size = backend.window_size();

    let mode = Mode {
        size,
        refresh: 60_000,
    };
    let output = Output::new(
        OUTPUT_NAME.to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".into(),
            model: "Winit".into(),
        },
    );

    Ok(())
}
