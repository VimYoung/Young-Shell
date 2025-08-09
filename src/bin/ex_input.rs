use std::{
    env,
    error::Error,
    sync::mpsc,
    sync::{Arc, RwLock},
};

use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, ForeignController, LayerAnchor, LayerType, WindowConf},
    wayland_adapter::SpellWin,
    Handle,
};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::new(
        376,
        576,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        BoardType::Exclusive,
        false,
    );
    let (waywindow, event_queue) = SpellWin::invoke_spell("counter-widget", window_conf);

    let ui = InputEx::new().unwrap();
    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    cast_spell::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        waywindow,
        event_queue,
        None,
        None,
        None,
    )
}
