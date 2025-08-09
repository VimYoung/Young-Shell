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
    let (tx, rx) = mpsc::channel::<Handle>();
    let window_conf = WindowConf::new(
        376,
        576,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        BoardType::None,
        false,
    );
    let (waywindow, event_queue) = SpellWin::invoke_spell("counter-widget", window_conf);

    let ui = Resize::new().unwrap();
    let mut small_size: bool = true;
    ui.on_request_increase_value(move || {
        if small_size {
            tx.send(Handle::Resize(0, 0, 376, 576)).unwrap();
            small_size = false;
        } else {
            small_size = true;
            tx.send(Handle::Resize(0, 0, 356, 546)).unwrap();
        }
    });
    cast_spell::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        waywindow,
        event_queue,
        Some(rx),
        None,
        None,
    )
}
