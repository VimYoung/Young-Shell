use std::{convert::TryInto, error::Error, num::NonZeroU32, rc::Rc};

use slint::{
    platform::software_renderer::RepaintBufferType::{self, SwappedBuffers},
    PhysicalSize, Rgb8Pixel, Window,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    output::{OutputHandler, OutputState},
    reexports::client::{globals::registry_queue_init, Connection, QueueHandle},
    registry::{ProvidesRegistryState, RegistryState},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        },
        WaylandSurface,
    },
    shm::{
        slot::{Buffer, SlotPool},
        Shm, ShmHandler,
    },
};

mod slint_adapter;
mod wayland_adapter;
use crate::{
    slint_adapter::{SlintLayerShell, SpellWinAdapter},
    wayland_adapter::WayWinAdapter,
};

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // Dimentions for the widget size
    let width = 256; //1366;
    let height = 256; //768;

    // let width = self.window_adapter.size.width;
    // let height = self.window_adapter.size.height;
    const DISPLAY_WIDTH: usize = 256;
    const DISPLAY_HEIGHT: usize = 256;

    let mut buffer1 = [Rgb8Pixel::new(0, 0, 0); DISPLAY_WIDTH * DISPLAY_HEIGHT];
    let mut buffer2 = [Rgb8Pixel::new(0, 0, 0); DISPLAY_WIDTH * DISPLAY_HEIGHT];

    //configure wayland to use these bufferes.
    let mut currently_displayed_buffer: &mut [_] = &mut buffer1;
    let mut work_buffer: &mut [_] = &mut buffer2;

    // Initialisation of wayland components.
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh: QueueHandle<WayWinAdapter> = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
    let surface = compositor.create_surface(&qh);

    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("simple_layer"), None);
    layer.set_anchor(Anchor::BOTTOM);
    // layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(width, height);
    layer.commit();
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

    let window = SpellWinAdapter::new(SwappedBuffers, width, height);
    let mut waywindow = WayWinAdapter::new(
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

    //Slint Managing Inputs;
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    // ui.run()?;

    println!("Casting the Spell");

    loop {
        slint::platform::update_timers_and_animations();
        // Following line does the updates to the buffer. Now those updates
        // needs to be picked by the compositer/windowing system and then
        // displayed accordingly.
        println!("Running the loop");
        window.rendered.render(work_buffer, DISPLAY_WIDTH);
        waywindow.set_buffer(currently_displayed_buffer.try_into()?);
        // println!("Ran till here.");
        event_queue.blocking_dispatch(&mut waywindow).unwrap();
        core::mem::swap::<&mut [_]>(&mut work_buffer, &mut currently_displayed_buffer);
    }

    // Ok(())
}
