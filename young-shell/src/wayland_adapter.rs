use std::{convert::TryInto, num::NonZeroU32};

use slint::Rgb8Pixel;
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::client::{
        protocol::{wl_output, wl_pointer, wl_shm, wl_surface},
        Connection, EventQueue, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};

use smithay_client_toolkit::{
    compositor::CompositorState,
    reexports::client::globals::registry_queue_init,
    shell::wlr_layer::{Anchor, Layer, LayerShell},
};

pub struct SpellWin {
    pub width: u32,
    pub height: u32,
    pub slint_buffer: Option<Vec<Rgb8Pixel>>,
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub shm: Shm,
    pub pool: SlotPool,
    pub layer: LayerSurface,
    pub keyboard_focus: bool,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub exit: bool,
    pub first_configure: bool,
}

impl SpellWin {
    fn new(
        width_height: (u32, u32),
        slint_buffer: Option<Vec<Rgb8Pixel>>,
        registry_state: RegistryState,
        // seat_state: SeatState,
        output_state: OutputState,
        // event_queue: EventQueue<Self>,
        shm: Shm,
        pool: SlotPool,
        layer: LayerSurface,
        keyboard_focus: bool,
        pointer: Option<wl_pointer::WlPointer>,
        exit: bool,
        first_configure: bool,
    ) -> Self {
        SpellWin {
            width: width_height.0,
            height: width_height.1,
            slint_buffer,
            registry_state,
            // seat_state,
            output_state,
            // event_queue,
            shm,
            pool,
            layer,
            keyboard_focus,
            pointer,
            exit,
            first_configure,
        }
    }

    pub fn invoke_spell<'a>(
        width: u32,
        height: u32,
        buffer1: &'a mut Vec<Rgb8Pixel>,
        buffer2: &'a mut Vec<Rgb8Pixel>,
    ) -> (
        Self,
        &'a mut [Rgb8Pixel],
        &'a mut [Rgb8Pixel],
        EventQueue<SpellWin>,
    ) {
        //configure wayland to use these bufferes.
        let currently_displayed_buffer: &mut [_] = buffer1;
        let work_buffer: &mut [_] = buffer2;

        // Initialisation of wayland components.
        let conn = Connection::connect_to_env().unwrap();
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh: QueueHandle<SpellWin> = event_queue.handle();

        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
        let surface = compositor.create_surface(&qh);

        let layer =
            layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("simple_layer"), None);
        layer.set_anchor(Anchor::BOTTOM);
        // layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer.set_size(width, height);
        layer.commit();
        let pool =
            SlotPool::new((width * height * 4) as usize, &shm).expect("Failed to create pool");

        return (
            SpellWin::new(
                (width, height),
                None,
                RegistryState::new(&globals),
                /*SeatState::new(&globals, &qh),*/ OutputState::new(&globals, &qh),
                shm,
                pool,
                layer,
                false,
                None,
                false,
                true,
            ),
            work_buffer,
            currently_displayed_buffer,
            event_queue,
        );
    }

    pub fn set_buffer(&mut self, buffer: Vec<Rgb8Pixel>) {
        self.slint_buffer = Some(buffer);
    }

    fn converter(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;
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
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(index, chunk)| {
                    let a: u8 = 0xFF;
                    let r = self.slint_buffer.as_ref().unwrap()[index].r;
                    let g = self.slint_buffer.as_ref().unwrap()[index].g;
                    let b = self.slint_buffer.as_ref().unwrap()[index].b;
                    let color: u32 =
                        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);

                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.to_le_bytes();
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

        // TODO save and reuse buffer when the window size is unchanged.  This is especially
        // useful if you do damage tracking, since you don't need to redraw the undamaged parts
        // of the canvas.
    }

    // fn initialise_application(&mut self, mut event_queue: EventQueue<Self>) {
    //     self.event_queue.blocking_dispatch(self).unwrap();
    // }
}

delegate_compositor!(SpellWin);
delegate_registry!(SpellWin);
delegate_output!(SpellWin);
delegate_shm!(SpellWin);
// delegate_seat!(SpellWin);
// delegate_keyboard!(SpellWin);
// delegate_pointer!(SpellWin);
delegate_layer!(SpellWin);

impl ShmHandler for SpellWin {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl OutputHandler for SpellWin {
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

impl CompositorHandler for SpellWin {
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

impl LayerShellHandler for SpellWin {
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
            self.converter(qh);
        }
    }
}

// impl SeatHandler for SpellWin {
//     fn seat_state(&mut self) -> &mut SeatState {
//         &mut self.seat_state
//     }
//
//     fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
//
//     fn new_capability(
//         &mut self,
//         _conn: &Connection,
//         qh: &QueueHandle<Self>,
//         seat: wl_seat::WlSeat,
//         capability: Capability,
//     ) {
//         if capability == Capability::Keyboard && self.keyboard.is_none() {
//             println!("Set keyboard capability");
//             let keyboard = self
//                 .seat_state
//                 .get_keyboard(qh, &seat, None)
//                 .expect("Failed to create keyboard");
//             self.keyboard = Some(keyboard);
//         }
//
//         if capability == Capability::Pointer && self.pointer.is_none() {
//             println!("Set pointer capability");
//             let pointer = self
//                 .seat_state
//                 .get_pointer(qh, &seat)
//                 .expect("Failed to create pointer");
//             self.pointer = Some(pointer);
//         }
//     }
//
//     fn remove_capability(
//         &mut self,
//         _conn: &Connection,
//         _: &QueueHandle<Self>,
//         _: wl_seat::WlSeat,
//         capability: Capability,
//     ) {
//         if capability == Capability::Keyboard && self.keyboard.is_some() {
//             println!("Unset keyboard capability");
//             self.keyboard.take().unwrap().release();
//         }
//
//         if capability == Capability::Pointer && self.pointer.is_some() {
//             println!("Unset pointer capability");
//             self.pointer.take().unwrap().release();
//         }
//     }
//
//     fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
// }
//
impl ProvidesRegistryState for SpellWin {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState /*, SeatState*/];
}
