use std::{
    any::Any,
    error::Error,
    sync::{mpsc, Arc, RwLock},
};

use spell::{
    enchant_spells,
    layer_properties::{
        BoardType, DataType, ForeignController, LayerAnchor, LayerType, WindowConf,
    },
    slint_adapter::{SpellMultiLayerShell, SpellMultiWinHandler},
    wayland_adapter::SpellWin,
    Handle,
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
    let (menu_tx, menu_rx) = mpsc::channel::<Handle>();
    let (bar_tx, bar_rx) = mpsc::channel::<Handle>();
    let mut is_menu_open = true;
    let windows_handler = SpellMultiWinHandler::new(vec![
        (
            "top-bar",
            WindowConf::new(
                1366,
                35,
                (Some(LayerAnchor::TOP), None),
                (0, 0, 0, 0),
                LayerType::Top,
                BoardType::None,
                true,
            ),
        ),
        (
            "menu",
            WindowConf::new(
                376,
                576,
                (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
                (5, 0, 0, 10),
                LayerType::Top,
                BoardType::None,
                false,
            ),
        ),
    ]);
    // let width: u32 = 376; //1366;
    // let height: u32 = 576; //768;

    let result_val = slint::platform::set_platform(Box::new(SpellMultiLayerShell {
        window_manager: windows_handler.clone(),
    }));
    if let Err(err_val) = result_val {
        panic!("{err_val}");
    }

    let bar = TopBar::new().unwrap();
    let menu = Menu::new().unwrap();
    let state = Box::new(menu.get_state());
    // let state = Box::new(ui.get_state());
    let menu_tx_clone = menu_tx.clone();
    let bar_tx_clone = bar_tx.clone();
    let ui_handle = menu.as_weak().unwrap();
    bar.on_request_menu_toggle(move || {
        menu_tx_clone.send(Handle::ToggleWindow).unwrap();
        if is_menu_open {
            is_menu_open = false;
            bar_tx_clone.send(Handle::RemoveKeyboardFocus).unwrap()
        } else {
            is_menu_open = true;
            bar_tx_clone.send(Handle::GrabKeyboardFocus).unwrap();
        }
    });
    bar.invoke_request_menu_toggle();

    let value = SpellWin::conjure_spells(windows_handler);

    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    // let val = bar.show();
    // if let Err(err_val) = val {
    //     println!("{err_val}");
    // }
    // cast_spell(waywindow, event_queue, rx, None, None)
    enchant_spells::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        value,
        vec![Some(bar_rx), Some(menu_rx)],
        vec![None, Some(Arc::new(RwLock::new(state)))],
        vec![
            None,
            Some(Box::new(
                |state_value: Arc<RwLock<Box<dyn ForeignController + 'static>>>| {
                    println!("Entered in the callback");
                    let controller_val = state_value.read().unwrap();
                    let inner = controller_val.as_ref();
                    let val = inner.as_any().downcast_ref::<State>().unwrap();
                    ui_handle.set_state(val.clone());
                },
            )),
        ],
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
