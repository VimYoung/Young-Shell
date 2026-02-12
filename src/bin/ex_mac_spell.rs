use slint::ComponentHandle;
use spell_framework::{
    self,
    layer_properties::{LayerAnchor, LayerType, WindowConf},
};
use std::{env, error::Error};
slint::include_modules!();
spell_framework::generate_widgets![AppWindow];

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::builder()
        .width(376_u32)
        .height(576_u32)
        .anchor_1(LayerAnchor::TOP)
        .margins(5, 0, 0, 10)
        .layer_type(LayerType::Top)
        .monitor("eDP-1".to_string())
        .build()
        .unwrap();
    println!("WindowConf = {:?}", window_conf);

    let ui = AppWindowSpell::invoke_spell("counter-widget", window_conf);
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    spell_framework::cast_spell!(ui)
}
