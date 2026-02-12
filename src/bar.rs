use std::{env, path::Path, process::Command, rc::Rc, thread};

use crate::{AppLineData, BarState, TopBarSpell};
use slint::{ComponentHandle, Image};
use spell_framework::{vault::AppSelector, wayland_adapter::WinHandle};

pub fn configure_bar(bar: &mut TopBarSpell, bar_tx: WinHandle) {
    let app_selector = AppSelector::default();
    let app_data_slint: Vec<AppLineData> = app_selector
        .get_primary()
        .map(|value| {
            let imag_path_val = if let Some(val) = value.image_path.clone() {
                val
            } else {
                "/home/ramayen/assets/kitty.png".to_string()
            };
            AppLineData {
                image: Image::load_from_path(Path::new(&imag_path_val)).unwrap_or(
                    Image::load_from_path(Path::new("/home/ramayen/assets/kitty.png")).unwrap(),
                ),
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
    bar.on_walls_window_called({
        let bar_handle = bar.as_weak().unwrap();
        move || {
            if !bar_handle.get_walls_open() {
                bar_tx_clone_a.add_input_region(0, 35, 1366, 315);
            } else {
                bar_tx_clone_a.subtract_input_region(0, 35, 1366, 315);
            }
        }
    });

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
                        image: Image::load_from_path(Path::new(&imag_path_val)).unwrap_or(
                            Image::load_from_path(Path::new("/home/ramayen/assets/kitty.png"))
                                .unwrap(),
                        ),
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
    println!("For loops set");

    let dark_walls_slint: Rc<slint::VecModel<Image>> = Rc::new(slint::VecModel::from(dark_walls));
    bar_handle.set_walls_paths(dark_walls_slint.into());

    let light_walls_slint: Rc<slint::VecModel<Image>> = Rc::new(slint::VecModel::from(light_walls));
    bar_handle.set_walls_light_paths(light_walls_slint.into());

    // bar.on_walls_window_called({ move || {} });

    bar.on_set_wallpaper(|img_path| {
        let img_path_str = img_path.path().unwrap().as_os_str().to_str().unwrap();
        println!("Image path : {}", img_path_str);
        let comm: String = if env::var("NIRI_SOCKET").is_ok() {
            String::from("swww img ") + "\"" + img_path_str + "\""
        } else {
            String::from("papermizer ") + img_path_str
        };
        println!("The command is :{}", comm);
        // let final_comm = Command::new(&sh).arg(c).arg(&comm);
        let mut final_comm = Command::new("sh");
        final_comm.arg("-c").arg(comm);
        final_comm.output().unwrap();
    });
}
