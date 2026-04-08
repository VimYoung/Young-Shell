use crate::{MainState, MenuSpell, MprisState};
use chrono::Local;
use image::{imageops::crop_imm, open};
use imageproc::filter::gaussian_blur_f32;
use slint::{ComponentHandle, Image, SharedString};
use spell_framework::{self, vault::mpris::PlayerFinder};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use sysinfo::{Components, CpuRefreshKind, RefreshKind, System};

pub fn configure_menu(menu: &mut MenuSpell) {
    let mut s =
        System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));

    // Wait a bit because CPU usage is based on diff.
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu_all();

    let player_finder = PlayerFinder::new().expect("Couldn't get mpris handler");
    menu.global::<MprisState>().on_refresh_mpris({
        let menu_weak = menu.as_weak();
        move || {
            if let Ok(player) = player_finder.find_active()
                && let Ok(metadata) = player.get_metadata()
                && let Some(title) = metadata.title()
            {
                let first_run = if menu_weak.unwrap().global::<MprisState>().get_first_run() {
                    menu_weak
                        .unwrap()
                        .global::<MprisState>()
                        .set_first_run(false);
                    menu_weak
                        .unwrap()
                        .global::<MprisState>()
                        .set_title(SharedString::from(title));
                    true
                } else {
                    false
                };
                if menu_weak
                    .unwrap()
                    .global::<MprisState>()
                    .get_title()
                    .as_str()
                    != title
                    || first_run
                {
                    let artist = metadata.artists().unwrap_or(vec!["Unknown"])[0].to_string();
                    let album = metadata.album_name().unwrap_or("Unknown").to_string();
                    let image_path = metadata
                        .art_url()
                        .unwrap_or("......./home/ramayen/assets/nomusic.png")
                        .to_string();
                    let image = Image::load_from_path(Path::new(&image_path[7..])).unwrap();
                    if let Ok(image_blur) = crop_blur_center(&image_path[7..], 326, 50) {
                        let image_blur_slint = Image::load_from_path(&image_blur).unwrap();
                        menu_weak
                            .unwrap()
                            .global::<MprisState>()
                            .set_song_poster_blur(image_blur_slint);

                        menu_weak
                            .unwrap()
                            .global::<MprisState>()
                            .set_artist(SharedString::from(artist));
                        menu_weak
                            .unwrap()
                            .global::<MprisState>()
                            .set_album(SharedString::from(album));
                        menu_weak
                            .unwrap()
                            .global::<MprisState>()
                            .set_song_poster(image);
                        menu_weak
                            .unwrap()
                            .global::<MprisState>()
                            .set_title(SharedString::from(title));
                    }
                }
            }
        }
    });
    menu.global::<MainState>().on_get_time({
        let menu_handle = menu.as_weak();
        move || {
            let now = Local::now();
            let time = now.format("%I:%M %p").to_string();
            menu_handle
                .unwrap()
                .global::<MainState>()
                .set_time(SharedString::from(time));
        }
    });

    menu.global::<MainState>().on_get_date({
        let menu_handle = menu.as_weak();
        move || {
            let now = Local::now();
            let date = now.format("%A %d, %b").to_string();
            menu_handle
                .unwrap()
                .global::<MainState>()
                .set_date(SharedString::from(date));
        }
    });

    menu.global::<MainState>().on_get_volume({
        let menu_handle = menu.as_weak();
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
            menu_handle
                .unwrap()
                .global::<MainState>()
                .set_vol(volume_int);
            // println!("Input text value {}", vol);
        }
    });

    menu.global::<MainState>().on_set_volume(move |volume_val| {
        std::process::Command::new("pactl")
            .args([
                "set-sink-volume",
                "@DEFAULT_SINK@",
                &format!("{}%", volume_val),
            ])
            .status()
            .unwrap();
    });

    menu.global::<MainState>().on_get_mic({
        let menu_handle = menu.as_weak();
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
            menu_handle.unwrap().global::<MainState>().set_mic(mic_int);
            // println!("Input text value {}", vol);
        }
    });

    menu.global::<MainState>().on_set_mic(move |mic_val| {
        std::process::Command::new("pactl")
            .args([
                "set-source-volume",
                "@DEFAULT_SOURCE@",
                &format!("{}%", mic_val),
            ])
            .status()
            .unwrap();
    });

    menu.global::<MainState>().on_get_bright({
        let menu_handle = menu.as_weak();
        move || {
            let output = std::process::Command::new("sh")
                .args(["-c", "brightnessctl -m | cut -d, -f4"])
                .output()
                .unwrap();

            let text = String::from_utf8_lossy(&output.stdout);
            let bright = text.trim().trim_end_matches('%');
            let bright_int = bright.parse::<i32>().unwrap();
            menu_handle
                .unwrap()
                .global::<MainState>()
                .set_brightness(bright_int);
        }
    });

    menu.global::<MainState>().on_set_bright(move |bright_val| {
        std::process::Command::new("brightnessctl")
            .args(["set", &format!("{}%", bright_val)])
            .status()
            .unwrap();
    });

    menu.global::<MainState>().on_get_cpu({
        let menu_handle = menu.as_weak();
        move || {
            let mut val: f32 = 0.0;
            s.refresh_cpu_all();
            for cpu in s.cpus() {
                val += cpu.cpu_usage();
            }
            let cpu_usage: f32 = val / (s.cpus().len() as f32);
            menu_handle
                .unwrap()
                .global::<MainState>()
                .set_cpu(cpu_usage);
        }
    });

    let mut components = Components::new_with_refreshed_list();
    menu.global::<MainState>().on_get_temp({
        let menu_handle = menu.as_weak();
        move || {
            let mut total: f32 = 0.0;
            for component in components.iter_mut() {
                component.refresh();
                total += component.temperature().unwrap_or_default();
            }
            let temp = total / components.len() as f32;
            menu_handle.unwrap().global::<MainState>().set_temp(temp);
        }
    });

    menu.on_call_close({
        let handle = menu.get_handler();
        move || {
            handle.remove_focus();
            handle.hide();
        }
    });

    menu.invoke_set_dark_theme();
}

pub fn crop_blur_center<P: AsRef<Path>>(
    input_path: P,
    target_width: u32,
    target_height: u32,
) -> Result<PathBuf, Box<dyn Error>> {
    // Load image
    let img = open(&input_path)?.to_rgba8();
    let (orig_w, orig_h) = img.dimensions();

    let target_ratio = target_width as f32 / target_height as f32;
    let orig_ratio = orig_w as f32 / orig_h as f32;

    // Determine crop dimensions
    let (crop_w, crop_h) = if orig_ratio > target_ratio {
        // Landscape / too wide → crop width
        let new_w = (orig_h as f32 * target_ratio).round() as u32;
        (new_w, orig_h)
    } else {
        // Too tall → crop height
        let new_h = (orig_w as f32 / target_ratio).round() as u32;
        (orig_w, new_h)
    };

    // Center offsets
    let x = (orig_w - crop_w) / 2;
    let y = (orig_h - crop_h) / 2;

    let cropped = crop_imm(&img, x, y, crop_w, crop_h).to_image();

    // Apply Gaussian blur (sigma = 8.0, adjust if needed)
    let blurred = gaussian_blur_f32(&cropped, 10.0);

    let output_path = PathBuf::from("/home/ramayen/assets/output-blur.png");

    // Remove existing file if it exists
    if output_path.exists() {
        fs::remove_file(&output_path)?;
    }

    // Save (this overwrites anyway, but we explicitly remove as requested)
    blurred.save(&output_path)?;

    Ok(output_path)
}
