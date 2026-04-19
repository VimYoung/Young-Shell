use chrono::Local;
use slint::{ComponentHandle, Image, SharedString};
use spell_framework::IpcController;
use spell_framework::vault::AppSelector;
use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
};
use std::{env, error::Error, path::Path, process::Command, rc::Rc, thread};
use sysinfo::{Components, CpuRefreshKind, RefreshKind, System};
slint::include_modules!();
spell_framework::generate_widgets![TopBar];

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::new(
        1536,
        610,
        (Some(LayerAnchor::TOP), None),
        (0, 0, 0, 0),
        LayerType::Top,
        BoardType::OnDemand,
        Some(30),
    );

    let mut bar = TopBarSpell::invoke_spell("top-bar", window_conf);
    bar.set_exclusive_zone(30);
    let bar_tx = bar.get_handler();
    configure_bar(&mut bar, bar_tx);
    cast_spell!(windows: [(bar, ipc)])
}

impl IpcController for TopBar {
    fn change_val(&self, _key: &str, _val: &str) {}

    fn get_type(&self, _key: &str) -> String {
        String::from("")
    }

    fn custom_command(&self, command: &str) {
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

fn configure_bar(bar: &mut TopBarSpell, bar_tx: WinHandle) {
    let app_selector = AppSelector::default();
    // println!("{:#?}", app_selector);
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
        println!("Application Running String.: {}", binding);
        if let Some((command, args)) = binding.split_once(' ') {
            command_val = command;
            args_vec = args.split(' ').collect();
        } else {
            command_val = &string_val;
        };
        let mut final_comm = Command::new("setsid");
        final_comm.arg(command_val);
        if !args_vec.is_empty() {
            args_vec.iter().for_each(|argument| {
                final_comm.arg(argument);
            });
        }
        final_comm.spawn().unwrap();
        println!("{string_val:?}");
    });

    let bar_tx_another = bar_tx.clone();
    let bar_tx_clone = bar_tx.clone();
    bar.on_search_toggle({
        move |search_toggle_value| {
            if search_toggle_value {
                bar_tx_another.grab_focus();
                bar_tx_clone.add_input_region(0, 35, 1536, 565);
            } else {
                bar_tx_another.remove_focus();
                bar_tx_clone.subtract_input_region(0, 35, 1536, 565);
            }
        }
    });
    bar.subtract_input_region(0, 35, 1536, 575);
    // let bar_tx_clone_a = bar_tx.clone();
    // let bar_tx_clone_b = bar_tx.clone();
    // bar_tx_clone_b.subtract_input_region(0, 35, 1366, 576);
    // bar.on_walls_window_called({
    //     let bar_handle = bar.as_weak().unwrap();
    //     move || {
    //         if !bar_handle.get_walls_open() {
    //             bar_tx_clone_a.add_input_region(0, 35, 1366, 315);
    //         } else {
    //             bar_tx_clone_a.subtract_input_region(0, 35, 1366, 315);
    //         }
    //     }
    // });
    bar.on_query_applications({
        let bar_handle = bar.as_weak().unwrap();
        move |query_value| {
            let app_data_native = app_selector.query_primary(query_value.as_ref(), 15);
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
    // // Commented for the sake of faster compilation
    // //
    // // let bar_handle = bar.as_weak().unwrap();
    // // let dark_wall_dir = Path::new("/home/ramayen/assets/wallpapers/");
    // // let light_wall_dir = Path::new("/home/ramayen/assets/light_walls/");
    // // let mut light_walls: Vec<Image> = Vec::new();
    // // let mut dark_walls: Vec<Image> = Vec::new();
    // //
    // // for inner_path in light_wall_dir.read_dir().expect("Couldn't read").flatten() {
    // //     if inner_path.path().is_file()
    // //         && (inner_path.path().extension().unwrap() == "png"
    // //             || inner_path.path().extension().unwrap() == "jpg"
    // //             || inner_path.path().extension().unwrap() == "jpeg")
    // //     {
    // //         light_walls.push(Image::load_from_path(&inner_path.path()).unwrap());
    // //     } else if inner_path.path().is_dir() {
    // //         for wall in inner_path
    // //             .path()
    // //             .read_dir()
    // //             .expect("Couldn't read")
    // //             .flatten()
    // //         {
    // //             if wall.path().is_file()
    // //                 && (wall.path().extension().unwrap() == "png"
    // //                     || wall.path().extension().unwrap() == "jpg"
    // //                     || wall.path().extension().unwrap() == "jpeg")
    // //             {
    // //                 light_walls.push(Image::load_from_path(&wall.path()).unwrap());
    // //             }
    // //         }
    // //     }
    // // }
    // // for inner_path in dark_wall_dir.read_dir().expect("Couldn't read").flatten() {
    // //     if inner_path.path().is_file()
    // //         && (inner_path.path().extension().unwrap() == "png"
    // //             || inner_path.path().extension().unwrap() == "jpg"
    // //             || inner_path.path().extension().unwrap() == "jpeg")
    // //     {
    // //         dark_walls.push(Image::load_from_path(&inner_path.path()).unwrap());
    // //     } else if inner_path.path().is_dir() {
    // //         for wall in inner_path
    // //             .path()
    // //             .read_dir()
    // //             .expect("Couldn't read")
    // //             .flatten()
    // //         {
    // //             if wall.path().is_file()
    // //                 && (wall.path().extension().unwrap() == "png"
    // //                     || wall.path().extension().unwrap() == "jpg"
    // //                     || wall.path().extension().unwrap() == "jpeg")
    // //             {
    // //                 dark_walls.push(Image::load_from_path(&wall.path()).unwrap());
    // //             }
    // //         }
    // //     }
    // // }
    // // println!("For loops set");
    // //
    // // let dark_walls_slint: Rc<slint::VecModel<Image>> = Rc::new(slint::VecModel::from(dark_walls));
    // // bar_handle.set_walls_paths(dark_walls_slint.into());
    // //
    // // let light_walls_slint: Rc<slint::VecModel<Image>> = Rc::new(slint::VecModel::from(light_walls));
    // // bar_handle.set_walls_light_paths(light_walls_slint.into());
    // //
    // // bar.on_walls_window_called({ move || {} });

    // bar.on_set_wallpaper(|img_path| {
    //     let img_path_str = img_path.path().unwrap().as_os_str().to_str().unwrap();
    //     println!("Image path : {}", img_path_str);
    //     let comm: String = if env::var("NIRI_SOCKET").is_ok() {
    //         String::from("swww img ") + "\"" + img_path_str + "\""
    //     } else {
    //         String::from("papermizer ") + img_path_str
    //     };
    //     println!("The command is :{}", comm);
    //     // let final_comm = Command::new(&sh).arg(c).arg(&comm);
    //     let mut final_comm = Command::new("sh");
    //     final_comm.arg("-c").arg(comm);
    //     final_comm.output().unwrap();
    // });

    bar.global::<MainState>().on_get_time({
        let menu_handle = bar.as_weak();
        move || {
            let now = Local::now();
            let time = now.format("%I:%M %p").to_string();
            menu_handle
                .unwrap()
                .global::<MainState>()
                .set_time(SharedString::from(time));
        }
    });

    bar.global::<MainState>().on_get_volume({
        let bar_handle = bar.as_weak();
        move || {
            let output = std::process::Command::new("pactl")
                .args(["get-sink-volume", "@DEFAULT_SINK@"])
                .output()
                .unwrap();

            let text = String::from_utf8_lossy(&output.stdout);
            let vol = text
                .split_whitespace()
                .find(|s| s.ends_with('%'))
                .unwrap()
                .trim_end_matches('%')
                .trim();
            let volume_int = vol.parse::<i32>().unwrap();
            bar_handle
                .unwrap()
                .global::<MainState>()
                .set_vol(volume_int);
            // println!("Input text value {}", vol);
        }
    });

    bar.global::<MainState>().on_set_volume(move |volume_val| {
        std::process::Command::new("pactl")
            .args([
                "set-sink-volume",
                "@DEFAULT_SINK@",
                &format!("{}%", volume_val),
            ])
            .status()
            .unwrap();
    });

    bar.global::<MainState>().on_get_mic({
        let bar_handle = bar.as_weak();
        move || {
            let output = std::process::Command::new("pactl")
                .args(["get-source-volume", "@DEFAULT_SOURCE@"])
                .output()
                .unwrap();

            let text = String::from_utf8_lossy(&output.stdout);
            let mic = text
                .split_whitespace()
                .find(|s| s.ends_with('%'))
                .unwrap()
                .trim_end_matches('%')
                .trim();
            let mic_int = mic.parse::<i32>().unwrap();
            bar_handle.unwrap().global::<MainState>().set_mic(mic_int);
        }
    });

    bar.global::<MainState>().on_set_mic(move |mic_val| {
        std::process::Command::new("pactl")
            .args([
                "set-source-volume",
                "@DEFAULT_SOURCE@",
                &format!("{}%", mic_val),
            ])
            .status()
            .unwrap();
    });

    let mut s =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));

    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu_all();
    bar.global::<MainState>().on_get_cpu({
        let bar_handle = bar.as_weak();
        move || {
            let mut val: f32 = 0.0;
            s.refresh_cpu_all();
            for cpu in s.cpus() {
                val += cpu.cpu_usage();
            }
            let cpu_usage: f32 = val / (s.cpus().len() as f32);
            bar_handle.unwrap().global::<MainState>().set_cpu(cpu_usage);
        }
    });

    // bar.on_toggle_inhibit(move |val| {
    //     if val {
    //         Command::new("vigiland").spawn().unwrap();
    //     } else {
    //         Command::new("pkill").arg("vigiland").output().unwrap();
    //     }
    // });
    //

    bar.global::<MainState>().on_get_bright({
        let bar_handle = bar.as_weak();
        move || {
            let output = std::process::Command::new("sh")
                .args(["-c", "brightnessctl -m | cut -d, -f4"])
                .output()
                .unwrap();

            let text = String::from_utf8_lossy(&output.stdout);
            let bright = text.trim().trim_end_matches('%');
            let bright_int = bright.parse::<i32>().unwrap();
            bar_handle
                .unwrap()
                .global::<MainState>()
                .set_brightness(bright_int);
        }
    });

    bar.on_refresh_battery({
        let bar_handle = bar.as_weak();
        move || {
            let output = std::process::Command::new("sh")
                .args(["-c", "acpi -b"])
                .output()
                .unwrap();
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some((before, _)) = text.rsplit_once(':') {
                let mut vals: Vec<&str> = before.split(", ").collect();
                vals.remove(0);
                let output_string = vals.join(" ");
                bar_handle
                    .unwrap()
                    .set_battery_val(SharedString::from(output_string));
            }
        }
    });
    let mut components = Components::new_with_refreshed_list();

    bar.global::<MainState>().on_get_temp({
        let bar_handle = bar.as_weak();
        move || {
            let mut total: f32 = 0.0;
            for component in components.iter_mut() {
                component.refresh();
                total += component.temperature().unwrap_or_default();
            }
            let temp = total / components.len() as f32;
            bar_handle.unwrap().global::<MainState>().set_temp(temp);
        }
    });
    // bar.on_refresh_temp({
    //     let bar_handle = bar.as_weak();
    //     move || {
    //    }
    // })
}
