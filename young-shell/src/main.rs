use std::{env, error::Error};

use spell::{
    cast_spell, get_spell_ingredients,
    layer_properties::{LayerAnchor, LayerType},
    slint_adapter::{SpellLayerShell, SpellWinAdapter},
    wayland_adapter::{SpellWin, WindowConf},
};

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("WAYLAND_DEBUG", "1");
    // env::set_var("RUST_BACKTRACE", "1");
    // Dimentions for the widget size
    let width: u32 = 376; //1366;
    let height: u32 = 576; //768;
    let window_adapter = SpellWinAdapter::new(width, height);
    let (mut buffer1, mut buffer2) = get_spell_ingredients(width, height);
    let window_conf = WindowConf::new(
        width,
        height,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        window_adapter.clone(),
        false,
    );

    let (waywindow, work_buffer, currently_displayed_buffer, event_queue) =
        SpellWin::invoke_spell("counter widget", &mut buffer1, &mut buffer2, window_conf);

    let platform_setting = slint::platform::set_platform(Box::new(SpellLayerShell {
        window_adapter: window_adapter.clone(),
    }));

    if let Err(error) = platform_setting {
        panic!("{error}");
    }
    let _ui = Menu::new()?;

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
    cast_spell(
        waywindow,
        window_adapter,
        event_queue,
        work_buffer,
        currently_displayed_buffer,
        width,
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
// TODO Making the Background transparent doesn't bring the contents of layer below it.
// This needs to be fixed
// TODO there is some off by one error happeneing if either of width and height is not
// a multiple of 4.
