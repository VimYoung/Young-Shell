use std::{
    env,
    error::Error,
    sync::mpsc,
    sync::{Arc, RwLock},
};

use slint::ComponentHandle;
use spell::{
    cast_spell,
    layer_properties::{ForeignController, LayerAnchor, LayerType, WindowConf},
    wayland_adapter::SpellWin,
    Handle,
};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let (_tx, rx) = mpsc::channel::<Handle>();
    let window_conf = WindowConf::new(
        376,
        576,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        false,
    );
    let (waywindow, event_queue) = SpellWin::invoke_spell("counter-widget", window_conf);

    let ui = AppWindow::new().unwrap();
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });
    cast_spell::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        waywindow,
        event_queue,
        rx,
        None,
        None,
    )
}
