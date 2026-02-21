use slint::ComponentHandle;
use std::{env, error::Error};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // let window_conf = WindowConf::builder()
    //     .width(376_u32)
    //     .height(576_u32)
    //     .anchor_1(LayerAnchor::TOP)
    //     .margins(5, 0, 0, 10)
    //     .layer_type(LayerType::Top)
    //     .build()
    //     .unwrap();
    // let menu = MenuSpell::invoke_spell(
    //     "menu",
    //     WindowConf::new(
    //         376,
    //         576,
    //         (Some(LayerAnchor::TOP), Some(LayerAnchor::RIGHT)),
    //         (5, 0, 0, 10),
    //         LayerType::Top,
    //         BoardType::None,
    //         None,
    //     ),
    // );
    let menu = Menu::new().unwrap();
    menu.invoke_set_dark_theme();
    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    //
    menu.run()?;
    Ok(())
    // cast_spell!(menu)
}
