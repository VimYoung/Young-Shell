use crate::{AppLineData, MainState, Menu, MenuFocus, TopBarSpell};
use chrono::Local;
use image::{imageops::crop_imm, open};
use imageproc::filter::gaussian_blur_f32;
use slint::{ComponentHandle, Image, Model, SharedString, Weak};
use spell_framework::{
    vault::{AppSelector, fuzzy_search_best_n},
    wayland_adapter::WinHandle,
};
use std::{
    env,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    thread,
};
use sysinfo::{Components, CpuRefreshKind, RefreshKind, System};

pub fn configure_bar(
    bar: &mut TopBarSpell,
    bar_tx: WinHandle,
    menu_tx: WinHandle,
    menu: Weak<Menu>,
) -> Vec<AppLineData> {
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

    let dock_apps: Vec<AppLineData> = app_data_slint
        .iter()
        .filter(|x| {
            ["Zed", "Dolphin", "Zen Browser", "Firefox", "Discord"].contains(&x.name.as_str())
        })
        .cloned()
        .collect();
    let vac_model = Rc::new(slint::VecModel::from(app_data_slint));
    bar.set_app_lines(vac_model.clone().into());
    bar.on_open_app(|string_val| {
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
        println!("{string_val:?}");
    });

    bar.on_refresh_clip({
        let bar_weak = bar.as_weak();
        move || {
            let clips = Command::new("cliphist")
                .arg("list")
                .output()
                .unwrap()
                .stdout;
            let clip_hist: Vec<SharedString> = clips
                .split(|&b| b == b'\n')
                .take(100)
                .map(|line| SharedString::from(std::str::from_utf8(line).unwrap_or_default()))
                .collect();
            let clip_model = Rc::new(slint::VecModel::from(clip_hist));
            bar_weak.unwrap().set_clip_lines(clip_model.clone().into());
        }
    });
    bar.on_search_toggle({
        let bar_tx_another = bar_tx.clone();
        let bar_tx_clone = bar_tx.clone();
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

    bar.on_query_clipboard({
        let bar_weak = bar.as_weak();
        move |query_val| {
            let x = bar_weak.unwrap().get_clip_lines().clone();
            let clips: Vec<String> = x.iter().map(|s| s.to_string()).collect();
            let string_refs: Vec<&str> = clips.iter().map(|s| s.as_str()).collect();
            let result: Vec<&str> = fuzzy_search_best_n(query_val.as_str(), &string_refs, 50)
                .iter()
                .map(|s| s.0)
                .collect();
            let result_clip_hist: Vec<SharedString> = result
                .iter()
                .map(|line| SharedString::from(*line))
                .collect();
            let clip_model = Rc::new(slint::VecModel::from(result_clip_hist));
            bar_weak.unwrap().set_clip_lines(clip_model.clone().into());
        }
    });

    bar.on_set_clip(move |val| {
        let mut decode = Command::new("cliphist")
            .arg("decode")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn `cliphist decode`");

        decode
            .stdin
            .take()
            .expect("Failed to open stdin for `cliphist decode`")
            .write_all(val.as_bytes())
            .expect("Failed to write to `cliphist decode` stdin");
        let decode_output = decode
            .wait_with_output()
            .expect("Failed to wait on `cliphist decode`");

        let mut wl_copy = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn `wl-copy`");

        // Write cliphist's output into wl-copy's stdin
        wl_copy
            .stdin
            .take()
            .expect("Failed to open stdin for `wl-copy`")
            .write_all(&decode_output.stdout)
            .expect("Failed to write to `wl-copy` stdin");

        wl_copy.wait().expect("Failed to wait on `wl-copy`");
        // if !wl_copy_status.success() {
        //     eprintln!("`wl-copy` exited with status: {}", wl_copy_status);
        //     std::process::exit(1);
        // }
    });

    bar.on_open_menu({
        let menu_txy = menu_tx.clone();
        let menu_weak = menu.clone();
        move |val| match val {
            MenuFocus::None => {
                menu_txy.toggle();
                menu_txy.grab_focus();
            }
            MenuFocus::Volume => {
                menu_weak.unwrap().set_current_focus(MenuFocus::Volume);
                menu_txy.toggle();
                menu_txy.grab_focus();
            }
            _ => {}
        }
    });

    let bar_handle = bar.as_weak().unwrap();

    let dark_wall_dir = Path::new("/home/ramayen/assets/wallpapers/");
    let light_wall_dir = Path::new("/home/ramayen/assets/light_walls/");
    let fallback = Path::new("/home/ramayen/assets/kitty.png");
    let dark_walls = collect_images(dark_wall_dir, fallback);
    let light_walls = collect_images(light_wall_dir, fallback);

    println!("Images loaded");

    let dark_walls_slint = Rc::new(slint::VecModel::from(dark_walls));
    bar_handle.set_walls_paths(dark_walls_slint.into());

    let light_walls_slint = Rc::new(slint::VecModel::from(light_walls));
    bar_handle.set_walls_light_paths(light_walls_slint.into());
    // bar.on_walls_window_called({ move || {} });

    bar.on_set_wallpaper(|img| {
        let img_path = img.path().unwrap().to_owned().clone();
        thread::spawn(move || {
            let img_path_str = img_path.as_os_str().to_str().unwrap();
            println!("Image path : {}", img_path_str);
            let blur_path = blur(img_path_str).unwrap();
            let sec_comm = "swaybg --image ".to_string() + blur_path.as_os_str().to_str().unwrap();
            println!("Image path blur: {}", sec_comm);
            if let Err(err) = copy_with_replace(&img_path) {
                println!("{}", err);
            }
            let comm: String = if env::var("NIRI_SOCKET").is_ok() {
                String::from("awww img --transition-type wipe ") + "\"" + img_path_str + "\""
            } else {
                String::from("papermizer ") + img_path_str
            };
            println!("The command is :{}", comm);
            // let final_comm = Command::new(&sh).arg(c).arg(&comm);
            let mut final_comm = Command::new("sh");
            final_comm.arg("-c").arg(comm);
            final_comm.output().unwrap();
            let mut blur_comm = Command::new("sh");
            blur_comm.arg("-c").arg(sec_comm);
            blur_comm.output().unwrap();
            // This needs to be handled properly
        });
    });

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

    bar.global::<MainState>().on_get_date({
        let bar_handle = bar.as_weak();
        move || {
            let now = Local::now();
            let date = now.format("%A %d, %b").to_string();
            bar_handle
                .unwrap()
                .global::<MainState>()
                .set_date(SharedString::from(date));
        }
    });
    // bar.on_refresh_temp({
    //     let bar_handle = bar.as_weak();
    //     move || {
    //    }
    // })
    //
    return dock_apps;
}

fn collect_images(dir: &Path, fallback: &Path) -> Vec<Image> {
    let mut images = Vec::new();

    fn visit(path: &Path, images: &mut Vec<Image>, fallback: &Path) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    visit(&path, images, fallback);
                } else {
                    let img = load_image_or_fallback(&path, fallback);
                    images.push(img);
                }
            }
        }
    }

    visit(dir, &mut images, fallback);
    images
}
fn is_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase()),
        Some(ref ext) if ext == "png" || ext == "jpg" || ext == "jpeg"
    )
}

fn load_image_or_fallback(path: &Path, fallback: &Path) -> Image {
    if path.exists() && path.is_file() && is_image(path) {
        if let Ok(img) = Image::load_from_path(path) {
            return img;
        } else {
            panic!("panicked here");
        }
    }
    Image::load_from_path(fallback).expect("Fallback image missing or invalid")
}

pub fn blur<P: AsRef<Path>>(input_path: P) -> Result<PathBuf, Box<dyn Error>> {
    // Load image
    let img = open(&input_path)?.to_rgba8();
    let blurred = gaussian_blur_f32(&img, 10.0);

    let output_path = PathBuf::from("/home/ramayen/output-wall-blur.png");

    // Remove existing file if it exists
    if output_path.exists() {
        fs::remove_file(&output_path)?;
    }

    // Save (this overwrites anyway, but we explicitly remove as requested)
    blurred.save(&output_path)?;

    Ok(output_path)
}

pub fn blur_lock<P: AsRef<Path>>(input_path: P) -> Result<(), Box<dyn Error>> {
    let img = open(&input_path)?.to_rgba8();
    let (orig_w, orig_h) = img.dimensions();
    let target_width = (orig_h as f32 / 2.5).min(orig_w as f32) as u32;
    let x = orig_w.saturating_sub(target_width);
    let cropped = crop_imm(&img, x, 0, target_width, orig_h).to_image();
    let blurred = gaussian_blur_f32(&cropped, 10.0);
    let output_path = PathBuf::from("/home/ramayen/assets/lock/wallpaper_blur.png");
    blurred.save(&output_path)?;
    Ok(())
}
fn copy_with_replace(src: &Path) -> std::io::Result<()> {
    let dest = Path::new("/home/ramayen/assets/lock/wallpaper.jpg");

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    println!("{:?}", dest);
    println!("{:?}", src);

    if dest.exists() {
        fs::remove_file(dest)?;
    }

    fs::copy(src, dest)?;

    // propagate error instead of swallowing
    if let Err(err) = blur_lock(dest) {
        println!("{}", err);
    }

    Ok(())
}
