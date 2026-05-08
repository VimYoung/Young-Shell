use std::{
    error::Error,
    process::{Command, Stdio},
    rc::Rc,
};
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
spell_framework::generate_widgets![TopBar, Menu, Workspaces, Dock];

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
    let dock = DockSpell::invoke_spell(
        "dock",
        WindowConf::builder()
            .width(420_u32)
            .height(80_u32)
            .anchor_1(LayerAnchor::BOTTOM)
            .margins(0, 0, -20, 0)
            .layer_type(LayerType::Top)
            .build()
            .unwrap(),
    );

    let bar_tx = bar.get_handler();
    let menu_tx = menu.get_handler();
    let menu_tx_another = menu_tx.clone();

    configure_menu(
        &mut menu,
        bar.as_weak().clone(),
        workspace.as_weak().clone(),
    );
    let dock_apps = configure_bar(&mut bar, bar_tx, menu_tx_another, menu.as_weak().clone());
    let dock_model = Rc::new(slint::VecModel::from(dock_apps));
    dock.set_docked_apps(dock_model.clone().into());
    // dock.subtract_input_region(0, 0, 420, 80);
    dock.on_hovered_on({
        let hx = dock.get_handler();
        move || {
            hx.add_input_region(0, 0, 420, 60);
        }
    });

    dock.on_hovered_off({
        let hx = dock.get_handler();
        move || {
            hx.subtract_input_region(0, 0, 420, 60);
        }
    });
    dock.on_open_app(|string_val| {
        let binding = string_val.to_string();
        let mut final_comm = Command::new("setsid");
        final_comm.arg("sh");
        final_comm.arg("-c");
        final_comm.arg(binding);
        println!("{:?}", final_comm);
        final_comm
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
    });
    configure_workpaces(&mut workspace);
    menu_tx.toggle();
    cast_spell!(windows: [menu, (bar,ipc), workspace, dock])
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
