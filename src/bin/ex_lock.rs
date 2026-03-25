use spell_framework::cast_spell;
use std::{env, error::Error, process::Command};

use slint::{ComponentHandle, SharedString};
use spell_framework::wayland_adapter::SpellLock;
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("SLINT_STYLE", "cosmic-dark");
    let lock = SpellLock::invoke_lock_spell();
    let lock_ui = LockScreen::new().unwrap();
    let looop_handle = lock.get_handler();
    lock_ui.on_check_pass({
        let lock_handle = lock_ui.as_weak();
        let looop_han = looop_handle.clone();
        move |string_val| {
            let lock_handle_a = lock_handle.clone().unwrap();
            let lock_handle_b = lock_handle.clone().unwrap();
            looop_han.unlock(
                None,
                string_val.to_string(),
                Box::new(move || {
                    lock_handle_a.set_lock_error(true);
                }),
                Box::new(move || {
                    lock_handle_b.set_is_lock_activated(false);
                }),
            );
        }
    });

    println!("Hello");
    lock_ui.on_verify_fingerprint({
        let lock_handle = lock_ui.as_weak();
        let loop_handle = looop_handle.clone();
        move || {
            println!("inside the function");
            let loock_handle = lock_handle.clone().unwrap();
            loop_handle.verify_fingerprint(Box::new(move || {
                loock_handle.set_finger_verification_response(SharedString::from("Failed"));
            }));
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
            let time = String::from_utf8(output.stdout).unwrap();
            // time = format!("{} {}", time.trim(), am_pm.trim());
            // println!("/{}/", time);
            lock_copy.set_time_var(time.into());
            lock_copy.set_time_ampm(am_pm.trim().into());
        }
    });
    // lock_ui.run()

    cast_spell!(lock: lock)
}
