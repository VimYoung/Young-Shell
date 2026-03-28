use std::error::Error;
mod bar;
mod menu;
mod workspace;
use bar::configure_bar;
use menu::configure_menu;
use spell_framework::{
    IpcController, cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
};
use workspace::configure_workpaces;

slint::include_modules!();
spell_framework::generate_widgets![TopBar, Menu, Workspaces];

fn main() -> Result<(), Box<dyn Error>> {
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
    let mut menu = MenuSpell::invoke_spell(
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

    configure_menu(&mut menu);
    configure_bar(&mut bar, bar_tx);
    configure_workpaces(&mut workspace);
    menu_tx.toggle();
    cast_spell!(windows: [menu, (bar,ipc), workspace])
}

impl IpcController for TopBar {
    fn change_val(&mut self, _key: &str, _val: &str) {}

    fn get_type(&self, _key: &str) -> String {
        String::from("")
    }

    fn custom_command(&mut self, command: &str) {
        match command {
            "toggle_search" => {
                if self.get_search_active() {
                    self.set_search_active(false);
                } else {
                    self.set_search_active(true);
                }
            }
            _ => {}
        }
    }
}
