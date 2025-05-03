use std::{convert::TryInto, error::Error, num::NonZeroU32, rc::Rc};

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

pub struct WayWinAdapter {
    pub width: u32,
    pub height: u32,
    pub slint_buffer: Option<[Rgb8Pixel; 256 * 256]>,
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

impl WayWinAdapter {
    pub fn new(
        width_height: (u32, u32),
        slint_buffer: Option<[Rgb8Pixel; 256 * 256]>,
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
        WayWinAdapter {
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

    pub fn set_buffer(&mut self, buffer: [Rgb8Pixel; 256 * 256]) {
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
                    let r = self.slint_buffer.unwrap()[index].r;
                    let g = self.slint_buffer.unwrap()[index].g;
                    let b = self.slint_buffer.unwrap()[index].b;
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

delegate_compositor!(WayWinAdapter);
delegate_registry!(WayWinAdapter);
delegate_output!(WayWinAdapter);
delegate_shm!(WayWinAdapter);
// delegate_seat!(WayWinAdapter);
// delegate_keyboard!(WayWinAdapter);
// delegate_pointer!(WayWinAdapter);
delegate_layer!(WayWinAdapter);

impl ShmHandler for WayWinAdapter {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl OutputHandler for WayWinAdapter {
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

impl CompositorHandler for WayWinAdapter {
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

impl LayerShellHandler for WayWinAdapter {
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

// impl SeatHandler for WayWinAdapter {
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
impl ProvidesRegistryState for WayWinAdapter {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState /*, SeatState*/];
}
