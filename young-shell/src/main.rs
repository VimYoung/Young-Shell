use std::error::Error;

use slint::{platform::software_renderer::RepaintBufferType::SwappedBuffers, Rgb8Pixel};
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{globals::registry_queue_init, Connection, QueueHandle},
    registry::RegistryState,
    shell::{
        wlr_layer::{Anchor, Layer, LayerShell},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm},
};

mod slint_adapter;
mod wayland_adapter;
use crate::{
    slint_adapter::{SlintLayerShell, SpellWinAdapter},
    wayland_adapter::SpellWin,
};

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // Dimentions for the widget size
    let width: u32 = 256; //1366;
    let height: u32 = 256; //768;
    let window = SpellWinAdapter::new(SwappedBuffers, width, height);
    let (mut buffer1, mut buffer2) = get_spell_ingredients(width, height);

    let (mut waywindow, mut work_buffer, mut currently_displayed_buffer, mut event_queue) =
        SpellWin::invoke_spell(width, height, &mut buffer1, &mut buffer2);
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

    println!("Casting the Spell");

    loop {
        slint::platform::update_timers_and_animations();
        // Following line does the updates to the buffer. Now those updates
        // needs to be picked by the compositer/windowing system and then
        // displayed accordingly.
        println!("Running the loop");
        window.rendered.render(work_buffer, width as usize);
        waywindow.set_buffer(work_buffer.to_vec());
        // println!("Ran till here.");
        event_queue.blocking_dispatch(&mut waywindow).unwrap();
        core::mem::swap::<&mut [_]>(&mut work_buffer, &mut currently_displayed_buffer);
    }
}

fn get_spell_ingredients(width: u32, height: u32) -> (Vec<Rgb8Pixel>, Vec<Rgb8Pixel>) {
    (
        vec![Rgb8Pixel::new(0, 0, 0); width as usize * height as usize],
        vec![Rgb8Pixel::new(0, 0, 0); width as usize * height as usize],
    )
}
