use std::error::Error;
mod bar;
mod workspace;
use bar::configure_bar;
use slint::{ComponentHandle, SharedString};
use spell_framework::{
    IpcController, cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
};
use workspace::configure_workpaces;

slint::include_modules!();
spell_framework::generate_widgets![TopBar, Menu, Workspaces];

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let mut bar = TopBarSpell::invoke_spell(
        "top-bar",
        WindowConf::new(
            1536,
            610,
            (Some(LayerAnchor::TOP), None),
            (0, 0, 0, 0),
            LayerType::Top,
            BoardType::None,
            Some(30),
        ),
    );
    let menu = MenuSpell::invoke_spell(
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
    );
    let mut workspace = WorkspacesSpell::invoke_spell(
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
    );
    // let bar = TopBar::new().unwrap();
    // let menu = Menu::new().unwrap();
    // let workspace = Workspaces::new().unwrap();
    // let [ref mut way_bar, ref mut way_menu, _] = windows[..] else {
    //     panic!("Error getting wayland handles");
    // };

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
    let bar_tx = bar.get_handler();
    let menu_tx = menu.get_handler();

    configure_bar(&mut bar, bar_tx);
    configure_workpaces(&mut workspace);
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
    cast_spell!(windows: [menu, bar, workspace])
}

// impl IpcController for TopBar {
//     fn get_type(&self, key: &str) -> String {
//         match key {
//             "is-search-on" => self.get_state().is_search_on.to_string(),
//             _ => String::from("Undocumented input"),
//         }
//     }
//     fn change_val(&mut self, key: &str, val: &str) {
//         match key {
//             "is-search-on" => self.set_state(BarState {
//                 is_search_on: val.parse::<bool>().unwrap(),
//             }),
//             _ => {}
//         }
//     }
// }

// impl IpcController for Menu {
//     fn get_type(&self, key: &str) -> String {
//         match key {
//             "is-power-menu-open" => self.get_state().is_power_menu_open.to_string(),
//             _ => String::from("Undocumented input"),
//         }
//     }

//     fn change_val(&mut self, key: &str, val: &str) {
//         let mut state = self.get_state();
//         match key {
//             "is-power-menu-open" => {
//                 state.is_power_menu_open = val.trim().parse::<bool>().unwrap();
//                 self.set_state(state);
//             }
//             "string-type" => {
//                 state.string_type = SharedString::from(val);
//                 self.set_state(state);
//             }
//             "enumsss" => println!("{:?}", state.cards_type),
//             _ => {}
//         }
//     }
// }
// TODO the cursor doesn't change from pointer to hand when clicking buttons, so the
// cursor needs to do that.
// TODO Lookup child creation in wayland, how can it be utilised.
// TODO Lookup popup in wayland to see if that helps in anything.
