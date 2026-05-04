use std::error::Error;
mod bar;
mod menu;
mod workspace;
use bar::configure_bar;
use menu::configure_menu;
use spell_framework::{
    IpcController, cast_spell,
    layer_properties::{LayerAnchor, LayerType, WindowConf},
};
use workspace::configure_workpaces;

slint::include_modules!();
spell_framework::generate_widgets![TopBar, Menu, Workspaces];

fn main() -> Result<(), Box<dyn Error>> {
    let mut bar = TopBarSpell::invoke_spell(
        "top-bar",
        WindowConf::builder()
            .width(1536_u32)
            .height(610_u32)
            .anchor_1(LayerAnchor::TOP)
            .layer_type(LayerType::Top)
            .exclusive_zone(30)
            .natural_scroll(true)
            .build()
            .unwrap(),
    );
    let mut menu = MenuSpell::invoke_spell(
        "menu",
        WindowConf::builder()
            .width(376_u32)
            .height(576_u32)
            .anchor_1(LayerAnchor::TOP)
            .anchor_2(LayerAnchor::LEFT)
            .layer_type(LayerType::Top)
            .build()
            .unwrap(),
    );
    let mut workspace = WorkspacesSpell::invoke_spell(
        "workspace",
        WindowConf::builder()
            .width(7_u32)
            .height(830_u32)
            .anchor_1(LayerAnchor::LEFT)
            .layer_type(LayerType::Top)
            .build()
            .unwrap(),
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
    let menu_tx_another = menu_tx.clone();

    configure_menu(
        &mut menu,
        bar.as_weak().clone(),
        workspace.as_weak().clone(),
    );
    configure_bar(&mut bar, bar_tx, menu_tx_another, menu.as_weak().clone());
    configure_workpaces(&mut workspace);
    menu_tx.toggle();
    cast_spell!(windows: [menu, (bar,ipc), workspace])
}

impl IpcController for TopBar {
    fn change_val(&self, _key: &str, _val: &str) {}

    fn get_type(&self, _key: &str) -> String {
        String::from("")
    }

    fn custom_command(&self, command: &str) {
        match command {
            "toggle_search" => {
                let val = !self.get_search_active();
                self.set_search_active(val);
            }
            "toggle_clip" => {
                self.set_selected(SelectedSection::Clipboard);
                self.set_search_active(true);
            }
            _ => {}
        }
    }
}
