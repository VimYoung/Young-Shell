use std::{any::Any, env, error::Error};

use slint::Rgb8Pixel;
use smithay_client_toolkit::shell::wlr_layer::{Anchor, Layer};

mod slint_adapter;
mod wayland_adapter;
use crate::{
    slint_adapter::{SlintLayerShell, SpellWinAdapter},
    wayland_adapter::SpellWin,
};
const WIDTH: usize = 256; //1366;
const HEIGHT: usize = 256; //768;

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("WAYLAND_DEBUG", "1");
    // Dimentions for the widget size
    // const WIDTH: u32 = 256; //1366;
    // const HEIGHT: u32 = 256; //768;

    // let (mut buffer1, mut buffer2) = get_spell_ingredients(width, height);

    let buffer1 = [Rgb8Pixel::new(0, 0, 0); WIDTH as usize * HEIGHT as usize];
    let buffer2 = [Rgb8Pixel::new(0, 0, 0); WIDTH as usize * HEIGHT as usize];

    let window_adapter = SpellWinAdapter::new(WIDTH as u32, HEIGHT as u32);

    let (mut waywindow, mut event_queue) = SpellWin::invoke_spell(
        "counter widget",
        buffer1,
        buffer2,
        Anchor::BOTTOM,
        Layer::Top,
        window_adapter.clone(),
    );

    let platform_setting = slint::platform::set_platform(Box::new(SlintLayerShell {
        window_adapter: window_adapter.clone(),
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
        // slint::platform::update_timers_and_animations();
        // Following line does the updates to the buffer. Now those updates
        // needs to be picked by the compositer/windowing system and then
        // displayed accordingly.
        // println!("Running the loop");
        // window_adapter.rendered.render(work_buffer, width as usize);
        // waywindow.set_buffer(currently_displayed_buffer.to_vec());
        if waywindow.first_configure {
            event_queue.roundtrip(&mut waywindow).unwrap();
        } else {
            event_queue.flush().unwrap();
            event_queue.dispatch_pending(&mut waywindow).unwrap();
            event_queue.blocking_dispatch(&mut waywindow).unwrap();
        }
    }
}

fn get_spell_ingredients(width: u32, height: u32) -> (Box<Vec<Rgb8Pixel>>, Box<Vec<Rgb8Pixel>>) {
    (
        Box::new(vec![
            Rgb8Pixel::new(0, 0, 0);
            width as usize * height as usize
        ]),
        Box::new(vec![
            Rgb8Pixel::new(0, 0, 0);
            width as usize * height as usize
        ]),
    )
}

// FIND explore the wayland popup feature and see if that can be leveraged to make
// connected/associated windows/menus.
// FIND window of slint is has a close method, that can be connected to surface
// remove/hide etc to close the window.
