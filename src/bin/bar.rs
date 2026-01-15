use std::{
    any::Any,
    env,
    error::Error,
    path::Path,
    process::Command,
    rc::Rc,
    sync::{Arc, RwLock},
    thread,
};

use slint::{ComponentHandle, Image};
use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, DataType, LayerAnchor, LayerType, WindowConf},
    wayland_adapter::SpellWin,
    ForeignController,
};
use spell_framework::{vault::AppSelector, wayland_adapter::WinHandle};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::new(
        1366,
        610,
        (Some(LayerAnchor::TOP), None),
        (0, 0, 0, 0),
        LayerType::Top,
        BoardType::None,
        Some(30),
    );

    let mut way_bar = SpellWin::invoke_spell("top-bar", window_conf);
    let bar = TopBar::new().unwrap();
    let bar_state = bar.get_state();
    let bar_handle = bar.as_weak().unwrap();
    way_bar.set_exclusive_zone(30);
    let bar_tx = way_bar.get_handler();
    configure_bar(bar, bar_tx);
    cast_spell(
        way_bar,
        Some(Arc::new(RwLock::new(bar_state))),
        Some(Box::new(move |state_value| {
            println!("Entered in the callback");
            let controller_val = state_value.read().unwrap();
            let val = controller_val.as_any().downcast_ref::<BarState>().unwrap();
            bar_handle.set_state(val.clone());
        })),
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

pub fn configure_bar(bar: TopBar, bar_tx: WinHandle) {
    let app_selector = AppSelector::default();
    let app_data_slint: Vec<AppLineData> = app_selector
        .get_primary()
        .map(|value| {
            let imag_path_val: String;
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
        let command_val: &str;
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
            if bar_handle.get_state().is_search_on {
                bar_handle.set_state(BarState { is_search_on: true });
                bar_tx_another.remove_focus();
                bar_tx_clone.subtract_input_region(0, 35, 1366, 576);
            } else {
                bar_handle.set_state(BarState {
                    is_search_on: false,
                });
                bar_tx_another.grab_focus();
                bar_tx_clone.add_input_region(0, 35, 1366, 576);
            }
        }
    });
    let bar_tx_clone_a = bar_tx.clone();
    let bar_tx_clone_b = bar_tx.clone();
    bar_tx_clone_b.subtract_input_region(0, 35, 1366, 576);
    bar.on_query_applications({
        let bar_handle = bar.as_weak().unwrap();
        move |query_value| {
            let app_data_native = app_selector.query_primary(query_value.as_ref(), 15);
            // println!("{:#?}", app_data_native);
            let app_data_slint: Vec<AppLineData> = app_data_native
                .iter()
                .map(|value| {
                    let imag_path_val: String;
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

    // bar.on_request_time(|| {
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
    let bar_handle = bar.as_weak().unwrap();
    let dark_wall_dir = Path::new("/home/ramayen/assets/wallpapers/");
    let light_wall_dir = Path::new("/home/ramayen/assets/light_walls/");
    let mut light_walls: Vec<Image> = Vec::new();
    let mut dark_walls: Vec<Image> = Vec::new();

    for inner_path in light_wall_dir.read_dir().expect("Couldn't read").flatten() {
        if inner_path.path().is_file()
            && (inner_path.path().extension().unwrap() == "png"
                || inner_path.path().extension().unwrap() == "jpg"
                || inner_path.path().extension().unwrap() == "jpeg")
        {
            light_walls.push(Image::load_from_path(&inner_path.path()).unwrap());
        } else if inner_path.path().is_dir() {
            for wall in inner_path
                .path()
                .read_dir()
                .expect("Couldn't read")
                .flatten()
            {
                if wall.path().is_file()
                    && (wall.path().extension().unwrap() == "png"
                        || wall.path().extension().unwrap() == "jpg"
                        || wall.path().extension().unwrap() == "jpeg")
                {
                    light_walls.push(Image::load_from_path(&wall.path()).unwrap());
                }
            }
        }
    }
    for inner_path in dark_wall_dir.read_dir().expect("Couldn't read").flatten() {
        if inner_path.path().is_file()
            && (inner_path.path().extension().unwrap() == "png"
                || inner_path.path().extension().unwrap() == "jpg"
                || inner_path.path().extension().unwrap() == "jpeg")
        {
            dark_walls.push(Image::load_from_path(&inner_path.path()).unwrap());
        } else if inner_path.path().is_dir() {
            for wall in inner_path
                .path()
                .read_dir()
                .expect("Couldn't read")
                .flatten()
            {
                if wall.path().is_file()
                    && (wall.path().extension().unwrap() == "png"
                        || wall.path().extension().unwrap() == "jpg"
                        || wall.path().extension().unwrap() == "jpeg")
                {
                    dark_walls.push(Image::load_from_path(&wall.path()).unwrap());
                }
            }
        }
    }

    let dark_walls_slint: Rc<slint::VecModel<Image>> = Rc::new(slint::VecModel::from(dark_walls));
    bar_handle.set_walls_paths(dark_walls_slint.into());

    let light_walls_slint: Rc<slint::VecModel<Image>> = Rc::new(slint::VecModel::from(light_walls));
    bar_handle.set_walls_light_paths(light_walls_slint.into());
    // bar.on_walls_window_called( || {} });

    bar.on_set_wallpaper(|img_path| {
        let img_path_str = img_path.path().unwrap().as_os_str().to_str().unwrap();
        println!("Image path: {}", img_path_str);
        let comm: String;
        if env::var("NIRI_SOCKET").is_ok() {
            comm = String::from("swww img ") + "\"" + img_path_str + "\"";
        } else {
            comm = String::from("swww img ") + "\"" + img_path_str + "\"";
            // comm = String::from("papermizer ") + img_path_str;
        }
        println!("The command is :{}", comm);
        // let final_comm = Command::new(&sh).arg(c).arg(&comm);
        let mut final_comm = Command::new("sh");
        final_comm.arg("-c").arg(comm);
        final_comm.output().unwrap();
    });
}
