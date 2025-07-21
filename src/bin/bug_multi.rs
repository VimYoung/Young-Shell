use std::{error::Error, sync::mpsc};

use slint::ComponentHandle;
use spell::{
    enchant_spells,
    layer_properties::{LayerAnchor, LayerType, WindowConf},
    slint_adapter::{SpellMultiLayerShell, SpellMultiWinHandler},
    wayland_adapter::SpellWin,
    Handle
};
slint::include_modules!();
fn main() -> Result<(), Box<dyn Error>> {
    let (bar_tx, bar_rx) = mpsc::channel::<Handle>();
    let windows_handler = SpellMultiWinHandler::new(vec![
        (
            "top-bar",
            WindowConf::new(
                1366,
                30,
                (Some(LayerAnchor::TOP), None),
                (0, 0, 0, 0),
                LayerType::Top,
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
    // let state = Box::new(ui.get_state());
    let bar_tx_clone = bar_tx.clone();
    bar.on_request_menu_hide(move || {
        bar_tx_clone.send(Handle::ToggleWindow).unwrap();
    });
    bar.invoke_request_menu_hide();

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
    enchant_spells(value, vec![None, Some(bar_rx)])
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
