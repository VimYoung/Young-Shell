use std::{
    any::Any,
    error::Error,
    sync::{Arc, RwLock},
};
mod bar;
mod workspace;
use spell_framework::{
    enchant_spells,
    layer_properties::{BoardType, DataType, LayerAnchor, LayerType, WindowConf},
    slint_adapter::SpellMultiWinHandler,
    ForeignController,
};
slint::include_modules!();
use bar::configure_bar;
use slint::ComponentHandle;
use workspace::configure_workpaces;
fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut windows = SpellMultiWinHandler::conjure_spells(vec![
        (
            "top-bar",
            WindowConf::new(
                1366,
                610,
                (Some(LayerAnchor::TOP), None),
                (0, 0, 0, 0),
                LayerType::Top,
                BoardType::None,
                Some(30),
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
                None,
            ),
        ),
        (
            "workspace",
            WindowConf::new(
                10,
                738,
                (Some(LayerAnchor::LEFT), None),
                (0, 0, 0, 0),
                LayerType::Top,
                BoardType::None,
                Some(10),
            ),
        ),
    ]);
    let bar = TopBar::new().unwrap();
    let menu = Menu::new().unwrap();
    let workspace = Workspaces::new().unwrap();
    let [ref mut way_bar, ref mut way_menu, _] = windows[..] else {
        panic!("Error getting wayland handles");
    };

    // way_bar.set_exclusive_zone(30);
    // way_bar.set_exclusive_zone(30);
    // forge.add_event(Duration::from_secs(2), move |_| {
    //     let output = Command::new("date")
    //         .args(["+%I:%M"])
    //         .output()
    //         .expect("failed to execute process");
    //
    //     let am_pm = String::from_utf8(
    //         Command::new("date")
    //             .args(["+%p"])
    //             .output()
    //             .expect("couldn't run")
    //             .stdout,
    //     )
    //     .unwrap();
    //     let mut time = String::from_utf8(output.stdout).unwrap();
    //     time = format!("{} {}", time.trim(), am_pm.trim());
    //     // println!("/{}/", time);
    //     bar_n.set_time_var(time.into());
    // });
    //
    let bar_tx = way_bar.get_handler();
    let menu_tx = way_menu.get_handler();
    let state = menu.get_state();
    let bar_state = bar.get_state();
    let menu_handle = menu.as_weak().unwrap();
    let bar_handle = bar.as_weak().unwrap();

    configure_bar(bar, bar_tx);
    configure_workpaces(workspace);
    menu_tx.toggle();
    // bar.global::<Rice>().on_get_volume(|| {
    //     let val = Command::new("sh").arg("-c").arg("pactl list sinks | grep '^[[:space:]]Volume:' | head -n $(( $SINK + 1 )) | tail -n 1 | sed -e 's,.* \\([0-9][0-9]*\\)%.*,\\1,'").output().unwrap();
    //     let output_str =  String::from_utf8(val.stderr).unwrap();
    //     println!("{}",output_str);
    //     output_str.into()
    // });
    //
    // let m = bar.as_weak().clone();
    // forge.add_event(Duration::from_secs(1), |_| {
    //     // bar.global::<Rice>().invoke_get_volume();
    //     m._walls_window_called();
    // });
    enchant_spells(
        windows,
        vec![
            Some(Arc::new(RwLock::new(bar_state))),
            Some(Arc::new(RwLock::new(state))),
            None,
        ],
        vec![
            Some(Box::new(move |state_value| {
                println!("Entered in the callback");
                let controller_val = state_value.read().unwrap();
                let val = controller_val.as_any().downcast_ref::<BarState>().unwrap();
                bar_handle.set_state(val.clone());
            })),
            Some(Box::new(move |state_value| {
                println!("Entered in the callback");
                let controller_val = state_value.read().unwrap();
                let val = controller_val.as_any().downcast_ref::<State>().unwrap();
                menu_handle.set_state(val.clone());
            })),
            None,
        ],
    )
}

impl ForeignController for BarState {
    fn get_type(&self, key: &str) -> DataType {
        match key {
            "is-search-on" => DataType::Boolean(self.is_search_on),
            _ => DataType::Panic,
        }
    }

    fn change_val(&mut self, key: &str, val: DataType) {
        if key == "is-search-on" {
            if let DataType::Boolean(value) = val {
                self.is_search_on = value;
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

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
// TODO the cursor doesn't change from pointer to hand when clicking buttons, so the
// cursor needs to do that.
// TODO Lookup child creation in wayland, how can it be utilised.
// TODO Lookup popup in wayland to see if that helps in anything.
