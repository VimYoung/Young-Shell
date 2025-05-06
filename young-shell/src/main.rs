use std::{cell::Cell, env, error::Error};

use slint::Rgb8Pixel;
use smithay_client_toolkit::shell::wlr_layer::{Anchor, Layer};

mod slint_adapter;
mod wayland_adapter;
use crate::{
    slint_adapter::{SlintLayerShell, SpellWinAdapter},
    wayland_adapter::SpellWin,
};

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("WAYLAND_DEBUG", "1");
    // Dimentions for the widget size
    let width: u32 = 400; //1366;
    let height: u32 = 400; //768;
    let window_adapter = SpellWinAdapter::new(width, height);
    let (mut buffer1, mut buffer2) = get_spell_ingredients(width, height);

    let (mut waywindow, mut work_buffer, mut currently_displayed_buffer, mut event_queue) =
        SpellWin::invoke_spell(
            "counter widget",
            width,
            height,
            &mut buffer1,
            &mut buffer2,
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
    let _ui = AppWindow::new().unwrap_or_else(|err| {
        panic!("{err}");
    });

    //Slint Managing Inputs;
    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    //
    // println!("Casting the Spell");

    loop {
        slint::platform::update_timers_and_animations();
        // Following line does the updates to the buffer. Now those updates
        // needs to be picked by the compositer/windowing system and then
        // displayed accordingly.
        // println!("Running the loop");
        if waywindow.render_again.replace(false) {
            window_adapter.draw_if_needed(|renderer| {
                // println!("Rendering");
                renderer.render(work_buffer, width as usize);
                waywindow.set_buffer(work_buffer.to_vec());
            });

            core::mem::swap::<&mut [_]>(&mut work_buffer, &mut currently_displayed_buffer);
        }
        if waywindow.first_configure {
            event_queue.roundtrip(&mut waywindow).unwrap();
        } else {
            event_queue.flush().unwrap();
            event_queue.dispatch_pending(&mut waywindow).unwrap();
            event_queue.blocking_dispatch(&mut waywindow).unwrap();
        }
    }
}

fn get_spell_ingredients(width: u32, height: u32) -> (Vec<Rgb8Pixel>, Vec<Rgb8Pixel>) {
    (
        vec![Rgb8Pixel::new(0, 0, 0); width as usize * height as usize],
        vec![Rgb8Pixel::new(0, 0, 0); width as usize * height as usize],
    )
}
// TODO the animations are jerky, you know the reason but you have to find a solution.
// TODO the cursor doesn't change from pointer to hand when clicking buttons, so the
// cursor needs to do that.
// TODO WlOutput is not properly implemented and managed. It is necessary for the proper
// functioning.
// TODO Lookup child creation in wayland, how can it be utilised.
// TODO Lookup popup in wayland to see if that helps in anything.
// TODO cursor shape management needs to be done.
