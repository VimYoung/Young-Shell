use std::{cell::RefCell, env, error::Error, rc::Rc};

use spell::{
    cast_spell, get_spell_ingredients,
    layer_properties::{LayerAnchor, LayerType, WindowConf},
    shared_context::SharedCore,
    skia_adapter::SpellSkiaWinAdapter,
    slint_adapter::SpellLayerShell,
    wayland_adapter::SpellWin,
};

slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("WAYLAND_DEBUG", "1");
    env::set_var("RUST_BACKTRACE", "1");
    // Dimentions for the widget size
    let width: u32 = 376; //1366;
    let height: u32 = 576; //768;
    let core = Rc::new(RefCell::new(SharedCore::new(width, height)));
    let window_adapter = SpellSkiaWinAdapter::new(core.clone(), width, height);
    // let work_buffer = get_spell_ingredients(width, height);
    let window_conf = WindowConf::new(
        width,
        height,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        core,
        window_adapter.clone(),
        false,
    );
    println!("Widnow Conf is set");

    let (waywindow, event_queue) = SpellWin::invoke_spell("counter-widget", window_conf);

    let platform_setting = slint::platform::set_platform(Box::new(SpellLayerShell {
        window_adapter,
        time_since_start: std::time::Instant::now(),
    }));
    println!("platform for slint is set");
    if let Err(error) = platform_setting {
        panic!("{error}");
    }
    let _ui = Menu::new()?;
    println!("ui's new window created");

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
    // ui.run()?;
    // Ok(())
    println!("spell is caseted");
    cast_spell(waywindow, event_queue)
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
