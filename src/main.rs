use std::{
    any::Any,
    env,
    error::Error,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::{mpsc, Arc, RwLock},
    thread,
};

use spell_framework::{
    enchant_spells,
    layer_properties::{
        BoardType, DataType, ForeignController, LayerAnchor, LayerType, WindowConf,
    },
    slint_adapter::{SpellMultiLayerShell, SpellMultiWinHandler},
    vault::AppSelector,
    wayland_adapter::SpellWin,
    Handle,
};
slint::include_modules!();
use slint::{ComponentHandle, Image, Model};

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
    // env::set_var("RUST_BACKTRACE", "full");
    let windows_handler = SpellMultiWinHandler::new(vec![
        (
            "top-bar",
            WindowConf::new(
                1366,
                610,
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
    slint::platform::set_platform(Box::new(SpellMultiLayerShell {
        window_manager: windows_handler.clone(),
    }))
    .unwrap();
    let bar = TopBar::new().unwrap();
    let menu = Menu::new().unwrap();
    let mut windows = SpellWin::conjure_spells(windows_handler);
    let [ref mut way_bar, ref mut way_menu] = windows[..] else {
        panic!("Error getting wayland handles");
    };
    let bar_tx = way_bar.get_handler();
    let menu_tx = way_menu.get_handler();
    let state = Box::new(menu.get_state());
    let ui_handle = menu.as_weak().unwrap();

    let app_selector = AppSelector::default();
    let app_data_slint: Vec<AppLineData> = app_selector
        .get_primary()
        .map(|value| {
            let mut imag_path_val = String::new();
            if let Some(val) = value.image_path.clone() {
                imag_path_val = val;
            } else {
                imag_path_val = "/home/ramayen/assets/kitty.png".to_string();
            }
            AppLineData {
                image: Image::load_from_path(Path::new(&imag_path_val))
                    .expect("Error loading image"),
                name: value.name.clone().into(),
                action: value
                    .exec_comm
                    .clone()
                    .unwrap_or_else(|| "no comm".to_string())
                    .into(),
            }
        })
        .collect();
    let vac_model = Rc::new(slint::VecModel::from(app_data_slint));
    bar.set_app_lines(vac_model.clone().into());
    bar.on_open_app(|string_val| {
        let mut command_val = "";
        let mut args_vec: Vec<&str> = Vec::new();
        let binding = string_val.to_string();
        if let Some((command, args)) = binding.split_once(' ') {
            command_val = command;
            args_vec = args.split(' ').collect();
        } else {
            command_val = &string_val;
        };
        let mut final_comm = Command::new(command_val);
        if !args_vec.is_empty() {
            args_vec.iter().for_each(|argument| {
                final_comm.arg(argument);
            });
        }
        thread::spawn(move || {
            final_comm.output().unwrap();
        });
        println!("{string_val:?}");
    });

    let bar_tx_another = bar_tx.clone();
    let bar_tx_clone = bar_tx.clone();
    bar.on_request_menu_toggle({
        let bar_handle = bar.as_weak().unwrap();
        move || {
            if bar_handle.get_is_search_on() {
                bar_handle.set_is_search_on(true);
                bar_tx_another.send(Handle::RemoveKeyboardFocus).unwrap();
                bar_tx_clone
                    .send(Handle::SubtractInputRegion(0, 35, 1366, 575))
                    .unwrap();
            } else {
                bar_handle.set_is_search_on(false);
                bar_tx_another.send(Handle::GrabKeyboardFocus).unwrap();
                bar_tx_clone
                    .send(Handle::AddInputRegion(0, 35, 1366, 575))
                    .unwrap();
            }
        }
    });
    let bar_tx_clone_a = bar_tx.clone();
    let bar_tx_clone_b = bar_tx.clone();
    bar_tx_clone_b
        .send(Handle::SubtractInputRegion(0, 35, 1366, 575))
        .unwrap();
    bar.on_walls_window_called({
        let bar_handle = bar.as_weak().unwrap();
        move || {
            if !bar_handle.get_walls_open() {
                bar_tx_clone_a
                    .send(Handle::AddInputRegion(0, 35, 1366, 315))
                    .unwrap();
            } else {
                bar_tx_clone_a
                    .send(Handle::SubtractInputRegion(0, 35, 1366, 315))
                    .unwrap();
            }
        }
    });

    bar.on_query_applications({
        let bar_handle = bar.as_weak().unwrap();
        move |query_value| {
            let app_data_slint: Vec<AppLineData> = app_selector
                .query_primary(query_value.as_ref(), 15)
                .iter()
                .map(|value| {
                    let mut imag_path_val = String::new();
                    if let Some(val) = value.image_path.clone() {
                        imag_path_val = val;
                    } else {
                        imag_path_val = "/home/ramayen/assets/kitty.png".to_string();
                    }
                    AppLineData {
                        image: Image::load_from_path(Path::new(&imag_path_val))
                            .expect("Error loading image"),
                        name: value.name.clone().into(),
                        action: value
                            .exec_comm
                            .clone()
                            .unwrap_or_else(|| "no comm".to_string())
                            .into(),
                    }
                })
                .collect();
            let vac_model = Rc::new(slint::VecModel::from(app_data_slint));
            bar_handle.set_app_lines(vac_model.clone().into());
        }
    });
    // app_display_tx.send(Handle::ToggleWindow).unwrap();
    menu_tx.send(Handle::ToggleWindow).unwrap();

    enchant_spells::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        windows,
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
