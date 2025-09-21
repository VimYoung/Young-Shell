use std::{
    any::Any,
    error::Error,
    sync::{Arc, RwLock},
};

use slint::ComponentHandle;
use spell_framework::{
    cast_spell,
    layer_properties::{
        BoardType, DataType, ForeignController, LayerAnchor, LayerType, WindowConf,
    },
    wayland_adapter::SpellWin,
};
slint::include_modules!();

impl ForeignController for State {
    fn get_type(&self, key: &str) -> DataType {
        match key {
            "is-power-menu-open" => DataType::Boolean(self.is_power_menu_open),
            _ => DataType::Panic,
        }
    }

    fn change_val(&mut self, key: &str, val: DataType) {
        match key {
            "is-power-menu-open" => {
                if let DataType::Boolean(value) = val {
                    self.is_power_menu_open = value;
                }
            }
            "string-type" => self.string_type = "hello".into(),
            "enumsss" => println!("{:?}", self.cards_type),
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "full");
    // Dimentions for the widget size
    // let width: u32 = 376; //1366;
    // let height: u32 = 576; //768;
    let window_conf = WindowConf::new(
        376,
        576,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        BoardType::None,
        false,
    );
    let waywindow = SpellWin::invoke_spell("menu", window_conf);

    let bar = TopBar::new().unwrap();
    let ui = Menu::new().unwrap();
    let state = Box::new(ui.get_state());

    let ui_clone = ui.as_weak().clone();

    let val = bar.show();
    if let Err(err_val) = val {
        println!("{err_val}");
    }
    cast_spell(
        waywindow,
        Some(Arc::new(RwLock::new(state))),
        Some(Box::new(
            move |state_value: Arc<RwLock<Box<dyn ForeignController>>>| {
                let controller_val = state_value.read().unwrap();
                let inner = controller_val.as_ref();
                let val = inner.as_any().downcast_ref::<State>().unwrap().clone();
                ui_clone.unwrap().set_state(val);
            },
        )),
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
// TODO there is some off by one error happeneing if either of width and height is not
// a multiple of 4.
