use core::{ffi::c_void, ptr::NonNull};
use i_slint_backend_winit::WinitWindowAccessor;
use raw_window_handle::{HasWindowHandle, RawWindowHandle, WaylandWindowHandle, WindowHandle};
use slint::{
    platform::{
        software_renderer::{RepaintBufferType, SoftwareRenderer},
        Platform, PlatformError, Renderer, WindowAdapter, WindowEvent,
    },
    BackendSelector, ComponentHandle, PhysicalSize, Window, WindowSize,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    globals::GlobalData,
    output::{OutputHandler, OutputState},
    reexports::{
        client::{
            backend::ObjectId,
            event_created_child,
            globals::{registry_queue_init, GlobalListContents},
            protocol::{
                wl_callback::{self, WlCallback},
                wl_compositor, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat,
                wl_surface,
            },
            Connection, Dispatch, Proxy, QueueHandle,
        },
        protocols::wp::input_method::zv1::client::{
            zwp_input_method_context_v1, zwp_input_method_v1, zwp_input_panel_surface_v1,
            zwp_input_panel_v1,
        },
        protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        // keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability,
        SeatHandler,
        SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure, LayerSurfaceData,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::{
    cell::Cell,
    error::Error,
    rc::{Rc, Weak},
    result::Result,
};
use std::{cell::RefCell, env};
use std::{/*convert::TryInto,*/ num::NonZeroU32};

// struct SlintLayerShell_Window;
// It is like equivalent to iced_layershell for slint.
// #[derive(Debug)]
struct SlintLayerShell {
    // For Platform implementation
    window: Rc<ShellWinAdapter>,
}

impl Platform for SlintLayerShell {
    fn create_window_adapter(&self) -> Result<Rc<(dyn WindowAdapter + 'static)>, PlatformError> {
        Ok(self.window.clone())
    }
}

/// This is a minimal adapter for a Window that doesn't have any other feature than rendering
/// using the software renderer.
pub struct ShellWinAdapter {
    window: Window,
    renderer: SoftwareRenderer,
    needs_redraw: Cell<bool>,
    size: Cell<PhysicalSize>,
}

impl ShellWinAdapter {
    /// The `repaint_buffer_type` parameter specify what kind of buffer are passed to the [`SoftwareRenderer`]
    pub fn new(repaint_buffer_type: RepaintBufferType) -> Rc<Self> {
        Rc::new_cyclic(|w: &Weak<Self>| Self {
            // window: Rc::new(RefCell::new(Window::new(w.clone()))),
            window: Window::new(w.clone()),
            renderer: SoftwareRenderer::new_with_repaint_buffer_type(repaint_buffer_type),
            needs_redraw: Default::default(),
            size: Default::default(),
        })
    }

    pub fn set_size(&self, size: impl Into<WindowSize>) {
        // self.window.borrow_mut().set_size(size);
        self.window.set_size(size);
    }

    fn is_winit_backend(&self) {
        if self.window.has_winit_window() {
            println!("the window has winit backend");
        } else {
            println!("the window doesn't have winit backend.");
        }
    }
}

impl WindowAdapter for ShellWinAdapter {
    fn window(&self) -> &Window {
        // let val = self.window.clone();
        return &self.window;
    }

    fn renderer(&self) -> &dyn Renderer {
        &self.renderer
    }

    fn size(&self) -> PhysicalSize {
        self.size.get()
    }

    fn set_size(&self, size: WindowSize) {
        self.size.set(size.to_physical(1.));
        self.window
            // .borrow_mut()
            .dispatch_event(WindowEvent::Resized {
                size: size.to_logical(1.),
            })
    }

    fn window_handle_06(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        // return self.window().window_handle().clone().window_handle();
        let surface: NonNull<c_void> = "Hello";
        let handle = WaylandWindowHandle::new(surface);
        let new_handle = RawWindowHandle::Wayland(handle);
        Ok(WindowHandle::borrow_raw(new_handle))
    }

    // fn display_handle_06(
    //     &self,
    // ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
    //     todo!()
    // }
    //
    fn request_redraw(&self) {
        self.needs_redraw.set(true);
    }
}

// impl core::ops::Deref for ShellWinAdapter {
//     type Target = Window;
//     fn deref(&self) -> &Self::Target {
//         &self.window
//     }
// }

struct ShellWin {
    // From Example of layer shell in smithay
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    // shm: Shm,
    exit: bool,
    first_configure: bool,
    // pool: SlotPool,
    width: u32,
    height: u32,
    shift: Option<u32>,
    layer: Option<LayerSurface>,
    // keyboard: Option<wl_keyboard::WlKeyboard>,
    // keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,
}

impl ShellWin {
    fn draw(&mut self, qh: QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        // let mut buffer1 = [Rgb565Pixel(0); DISPLAY_WIDTH * DISPLAY_HEIGHT];
        // let mut buffer2 = [Rgb565Pixel(0); DISPLAY_WIDTH * DISPLAY_HEIGHT];

        // Damage the whole for the purpose of redraw.
        if let Some(layer) = &self.layer {
            layer
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);

            layer
                .wl_surface()
                .frame(&qh, self.layer.clone().unwrap().wl_surface().clone());

            layer.commit();
        }

        // TODO save and reuse buffer when the window size is unchanged.  This is especially
        // useful if you do damage tracking, since you don't need to redraw the undamaged parts
        // of the canvas.

        // TODO I think that surfaace should be implemented here since it is what runs in loop
        // but since there is lack of things, I will do it in main, if that doesn't update, then
        // I will move that to here and try again.
    }
}

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "full");

    // let selector = BackendSelector::new().backend_name("winit".to_string());

    // if let Err(err) = selector.select() {
    //     println!("Error in selecting winit Backend: {err}");
    // } else {
    //     println!("Winit Backend Selected.");
    // }
    let window = ShellWinAdapter::new(Default::default());

    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh: QueueHandle<ShellWin> = event_queue.handle();

    slint::platform::set_platform(Box::new(SlintLayerShell {
        window: window.clone(),
    }))
    .unwrap();

    window.is_winit_backend();

    // let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    // let surface = compositor.create_surface(&qh);

    //slint part to get surface.
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    let ui = ui_handle.unwrap();
    let handle_binding = ui.window().window_handle();
    let handle = handle_binding.window_handle();

    {
        match handle {
            Ok(handle) => match handle.as_raw() {
                raw_window_handle::RawWindowHandle::Wayland(wayland_window) => {
                    let nn_surface = wayland_window.surface;
                    let wl_surface_obj_id: ObjectId;
                    unsafe {
                        wl_surface_obj_id = ObjectId::from_ptr(
                            wl_surface::WlSurface::interface(),
                            nn_surface.as_ptr().cast(),
                        )
                        .unwrap();
                    }
                    let wl_surface: wl_surface::WlSurface =
                        wl_surface::WlSurface::from_id(&conn, wl_surface_obj_id).unwrap();

                    let layer = layer_shell.create_layer_surface(
                        &qh,
                        wl_surface,
                        Layer::Top,
                        Some("simple_layer"),
                        None,
                    );
                    // Configure the layer surface, providing things like the anchor on screen, desired size and the keyboard
                    // interactivity
                    layer.set_anchor(Anchor::BOTTOM);
                    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
                    layer.set_size(256, 256);

                    //initial commit without anything attached.
                    layer.commit();

                    let mut simple_layer = ShellWin {
                        // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
                        // listen for seats and outputs.
                        registry_state: RegistryState::new(&globals),
                        seat_state: SeatState::new(&globals, &qh),
                        output_state: OutputState::new(&globals, &qh),
                        exit: false,
                        first_configure: true,
                        width: 256,
                        height: 256,
                        shift: None,
                        layer: Some(layer),
                        // keyboard: None,
                        // keyboard_focus: false,
                        pointer: None,
                    };

                    loop {
                        event_queue.blocking_dispatch(&mut simple_layer).unwrap();

                        if simple_layer.exit {
                            println!("exiting example");
                            break;
                        }
                    }
                }

                _ => {
                    println!("Some error occured");
                }
            },
            Err(e) => {
                println!("Failed to get Sling UI window handle!\n{e:?}, {e}")
            }
        }
    }

    println!("hello!, I am going to make this work.");
    Ok(())
    // There is going to be a ShellWin.enchnat()! for like ui.run() functionality.
}

impl PointerHandler for ShellWin {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            // Ignore events for other surfaces
            // if &event.surface != self.layer.wl_surface() {
            //     continue;
            // }
            match event.kind {
                Enter { .. } => {
                    println!("Pointer entered @{:?}", event.position);
                }
                Leave { .. } => {
                    println!("Pointer left");
                }
                Motion { .. } => {}
                Press { button, .. } => {
                    println!("Press {:x} @ {:?}", button, event.position);
                    self.shift = self.shift.xor(Some(0));
                }
                Release { button, .. } => {
                    println!("Release {:x} @ {:?}", button, event.position);
                }
                Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                }
            }
        }
    }
}

impl SeatHandler for ShellWin {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
        // if capability == Capability::Keyboard && self.keyboard.is_none() {
        //     println!("Set keyboard capability");
        //     let keyboard = self
        //         .seat_state
        //         .get_keyboard(qh, &seat, None)
        //         .expect("Failed to create keyboard");
        //     self.keyboard = Some(keyboard);
        // }

        // if capability == Capability::Pointer && self.pointer.is_none() {
        //     println!("Set pointer capability");
        //     let pointer = self
        //         .seat_state
        //         .get_pointer(qh, &seat)
        //         .expect("Failed to create pointer");
        //     self.pointer = Some(pointer);
        //}
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        // if capability == Capability::Keyboard && self.keyboard.is_some() {
        //     println!("Unset keyboard capability");
        //     self.keyboard.take().unwrap().release();
        // }

        if capability == Capability::Pointer && self.pointer.is_some() {
            println!("Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl CompositorHandler for ShellWin {
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
        self.draw(qh.clone());
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for ShellWin {
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

impl LayerShellHandler for ShellWin {
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
        self.width = NonZeroU32::new(configure.new_size.0).map_or(256, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(256, NonZeroU32::get);

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh.clone());
        }
    }
}
delegate_compositor!(ShellWin);
delegate_output!(ShellWin);
// delegate_shm!(ShellWin);
//
delegate_seat!(ShellWin);
// delegate_keyboard!(ShellWin);
delegate_pointer!(ShellWin);
delegate_layer!(ShellWin);
delegate_registry!(ShellWin);

impl ProvidesRegistryState for ShellWin {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
// Keyboard yet to implement.
