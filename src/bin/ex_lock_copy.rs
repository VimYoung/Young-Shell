use spell_framework::cast_spell;
use std::{
    env,
    error::Error,
    process::Command,
    sync::{Arc, RwLock},
};

use slint::{platform::PlatformError, ComponentHandle};
use spell_framework::{wayland_adapter::SpellLock, ForeignController};
slint::include_modules!();

fn main() -> Result<(), PlatformError> {
    env::set_var("SLINT_STYLE", "cosmic-dark");
    // let lock = SpellLock::invoke_lock_spell();
    let lock_ui = LockScreen::new().unwrap();
    let _menu = Menu::new().unwrap();
    // let looop_handle = lock.get_handler();
    lock_ui.on_check_pass({
        let lock_handle = lock_ui.as_weak();
        move |string_val| {
            let lock_handle_a = lock_handle.clone().unwrap();
            let lock_handle_b = lock_handle.clone().unwrap();
            // looop_handle.unlock(
            //     None,
            //     string_val.to_string(),
            //     Box::new(move || {
            //         lock_handle_a.set_lock_error(true);
            //     }),
            //     Box::new(move || {
            //         lock_handle_b.set_is_lock_activated(false);
            //     }),
            // );
        }
    });
    lock_ui.set_is_lock_activated(true);

    lock_ui.on_request_time({
        let lock_copy = lock_ui.as_weak().unwrap();
        move || {
            let output = Command::new("date")
                .args(["+%I:%M"])
                .output()
                .expect("failed to execute process");

            let am_pm = String::from_utf8(
                Command::new("date")
                    .args(["+%p"])
                    .output()
                    .expect("couldn't run")
                    .stdout,
            )
            .unwrap();
            let time_var = String::from_utf8(output.stdout).unwrap();
            println!("{}, {}", time_var.trim(), am_pm);
            // time = format!("{} {}", time.trim(), am_pm.trim());
            // println!("/{}/", time);
            lock_copy.set_time_var(time_var.trim().into());
            lock_copy.set_time_ampm(am_pm.trim().into());
        }
    });
    lock_ui.run()

    // eprintln!("Ran till here");
    // cast_spell(
    //     lock,
    //     None,
    //     None::<fn(Arc<RwLock<Box<dyn ForeignController>>>)>,
    // )
}
