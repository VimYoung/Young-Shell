use std::{cell::RefCell, convert::TryInto, error::Error, num::NonZeroU32, rc::Rc};

use slint::{
    platform::{
        software_renderer::{
            RepaintBufferType::{self, SwappedBuffers},
            SoftwareRenderer,
        },
        Platform, WindowAdapter,
    },
    PhysicalSize, Rgb8Pixel, Window,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::client::{
        globals::registry_queue_init,
        protocol::{wl_output, wl_pointer, wl_shm, wl_surface},
        Connection, EventQueue, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{
        slot::{Buffer, SlotPool},
        Shm, ShmHandler,
    },
};

struct SpellWindowInner {
    rendered: SoftwareRenderer,
    slint_buffer: [Rgb8Pixel; 256 * 256],
}

pub struct SpellWindowAdapter {
    window: Window,
    inner: RefCell<SpellWindowInner>,
    size: PhysicalSize,
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,
    exit: bool,
    first_configure: bool,
}

impl SpellWindowAdapter {
    fn new(
        repaint_buffer_type: RepaintBufferType,
        width: u32,
        height: u32,
        slint_buffer: [Rgb8Pixel; 256 * 256],
        registry_state: RegistryState,
        output_state: OutputState,
        shm: Shm,
        pool: SlotPool,
        layer: LayerSurface,
        keyboard_focus: bool,
        pointer: Option<wl_pointer::WlPointer>,
        exit: bool,
        first_configure: bool,
    ) -> Rc<Self> {
        Rc::<SpellWindowAdapter>::new_cyclic(|adapter| {
            let window = Window::new(adapter.clone());
            SpellWindowAdapter {
                window,
                inner: RefCell::new(SpellWindowInner {
                    rendered: SoftwareRenderer::new_with_repaint_buffer_type(repaint_buffer_type),
                    slint_buffer,
                }),
                size: PhysicalSize { width, height },
                registry_state,
                output_state,
                shm,
                pool,
                layer,
                keyboard_focus,
                pointer,
                exit,
                first_configure,
            }
        })
    }

    fn converter(&mut self, qh: &QueueHandle<Self>) {
        let width = self.size.width;
        let height = self.size.height;
        let stride = self.size.width as i32 * 4;
        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");
        // Drawing the window
        {
            let inner = self.inner.borrow();
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(index, chunk)| {
                    if index < inner.slint_buffer.len() {
                        let a: u8 = 0xFF;
                        let r = inner.slint_buffer[index].r;
                        let g = inner.slint_buffer[index].g;
                        let b = inner.slint_buffer[index].b;
                        let color: u32 = ((a as u32) << 24)
                            | ((r as u32) << 16)
                            | ((g as u32) << 8)
                            | (b as u32);

                        let array: &mut [u8; 4] = chunk.try_into().unwrap();
                        *array = color.to_le_bytes();
                    }
                });
        }

        // Damage the entire window
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }
}

impl WindowAdapter for SpellWindowAdapter {
    fn window(&self) -> &Window {
        &self.window
    }

    fn size(&self) -> PhysicalSize {
        PhysicalSize {
            width: self.size.width,
            height: self.size.height,
        }
    }

    fn renderer(&self) -> &dyn slint::platform::Renderer {
        // This is safe because we're ensuring the lifetime of the reference doesn't exceed
        // the lifetime of the borrow
        &self.inner.borrow().rendered
    }
}

struct SlintLayerShell {
    window_adapter: Rc<SpellWindowAdapter>,
}

impl Platform for SlintLayerShell {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window_adapter.clone())
    }
}

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // Dimensions for the widget size
    let width = 256;
    let height = 256;

    const DISPLAY_WIDTH: usize = 256;
    const DISPLAY_HEIGHT: usize = 256;

    let mut buffer1 = [Rgb8Pixel::new(0, 0, 0); DISPLAY_WIDTH * DISPLAY_HEIGHT];
    let mut buffer2 = [Rgb8Pixel::new(0, 0, 0); DISPLAY_WIDTH * DISPLAY_HEIGHT];

    // Configure wayland to use these buffers
    let mut currently_displayed_buffer: &mut [_] = &mut buffer1;
    let mut work_buffer: &mut [_] = &mut buffer2;

    // Initialisation of wayland components
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh: QueueHandle<SpellWindowAdapter> = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    let surface = compositor.create_surface(&qh);

    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("simple_layer"), None);
    layer.set_anchor(Anchor::BOTTOM);
    layer.set_size(width, height);
    layer.commit();
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

    let window = SpellWindowAdapter::new(
        SwappedBuffers,
        width,
        height,
        currently_displayed_buffer.try_into()?,
        RegistryState::new(&globals),
        OutputState::new(&globals, &qh),
        shm,
        pool,
        layer,
        false,
        None,
        false,
        true,
    );

    let platform_setting = slint::platform::set_platform(Box::new(SlintLayerShell {
        window_adapter: window.clone(),
    }));

    if let Err(error) = platform_setting {
        panic!("{error}");
    }
    let ui = AppWindow::new().unwrap_or_else(|err| {
        panic!("{err}");
    });

    // Slint Managing Inputs
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    println!("Starting the event loop");

    // Create a separate mutable reference to handle the Rc
    let window_handle = Rc::new(RefCell::new(window));

    loop {
        slint::platform::update_timers_and_animations();

        // Fix for line 267 - properly access the inner renderer with mutable borrow
        {
            let window_ref = window_handle.borrow();
            let mut inner = window_ref.inner.borrow_mut();
            inner.rendered.render(work_buffer, DISPLAY_WIDTH);
        }

        // Handle event queue with proper borrowing
        {
            let mut window_ref = window_handle.borrow_mut();
            event_queue.blocking_dispatch(&mut *window_ref).unwrap();
        }

        // Swap buffers
        core::mem::swap::<&mut [_]>(&mut work_buffer, &mut currently_displayed_buffer);

        // Update the buffer in window's inner state
        {
            let window_ref = window_handle.borrow();
            let mut inner = window_ref.inner.borrow_mut();
            // Only update if conversion succeeds
            if let Ok(buffer) = currently_displayed_buffer.try_into() {
                inner.slint_buffer = buffer;
            }
        }
    }
}

delegate_compositor!(SpellWindowAdapter);
delegate_registry!(SpellWindowAdapter);
delegate_output!(SpellWindowAdapter);
delegate_shm!(SpellWindowAdapter);
delegate_layer!(SpellWindowAdapter);

impl ShmHandler for SpellWindowAdapter {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl OutputHandler for SpellWindowAdapter {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl CompositorHandler for SpellWindowAdapter {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.converter(qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }
}

impl LayerShellHandler for SpellWindowAdapter {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.size.width = NonZeroU32::new(configure.new_size.0).map_or(256, NonZeroU32::get);
        self.size.height = NonZeroU32::new(configure.new_size.1).map_or(256, NonZeroU32::get);

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.converter(qh);
        }
    }
}

impl ProvidesRegistryState for SpellWindowAdapter {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
