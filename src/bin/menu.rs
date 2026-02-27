use image::{imageops::crop_imm, open};
use imageproc::filter::gaussian_blur_f32;
use slint::{ComponentHandle, Image, SharedString};
use spell_framework::{
    self, cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
    vault::mpris::PlayerFinder,
};
use std::fs;
use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};
slint::include_modules!();
spell_framework::generate_widgets![Menu];

fn main() -> Result<(), Box<dyn Error>> {
    // let window_conf = WindowConf::builder()
    //     .width(376_u32)
    //     .height(576_u32)
    //     .anchor_1(LayerAnchor::TOP)
    //     .margins(5, 0, 0, 10)
    //     .layer_type(LayerType::Top)
    //     .build()
    //     .unwrap();
    let menu = MenuSpell::invoke_spell(
        "menu",
        // WindowConf::builder()
        WindowConf::new(
            376,
            576,
            (Some(LayerAnchor::TOP), Some(LayerAnchor::RIGHT)),
            (5, 0, 0, 10),
            LayerType::Top,
            BoardType::None,
            None,
        ),
    );

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
    menu.invoke_set_dark_theme();
    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    //
    cast_spell!(menu)
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
