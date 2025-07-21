use std::{
    env,
    error::Error,
    sync::mpsc,
    sync::{Arc, RwLock},
};

use spell::{
    cast_spell,
    layer_properties::{ForeignController, LayerAnchor, LayerType, WindowConf},
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
        false,
    );
    let (waywindow, event_queue) = SpellWin::invoke_spell("counter-widget", window_conf);

    let ui = HideWin::new().unwrap();
    ui.on_request_hide(move || {
        tx.send(Handle::HideWindow).unwrap();
    });

    cast_spell::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        waywindow,
        event_queue,
        rx,
        None,
        None,
    )
}
