use spell_framework::cast_spell;
use std::{
    env,
    error::Error,
    sync::{Arc, RwLock},
};

use slint::ComponentHandle;
use spell_framework::{layer_properties::ForeignController, wayland_adapter::SpellLock};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("SLINT_STYLE", "cosmic-dark");
    let lock = SpellLock::invoke_lock_spell();
    let lock_ui = LockScreen::new().unwrap();
    let looop_handle = lock.get_handler();
    lock_ui.on_check_pass({
        let lock_handle = lock_ui.as_weak();
        move |string_val| {
            let lock_handle_a = lock_handle.clone().unwrap();
            let lock_handle_b = lock_handle.clone().unwrap();
            looop_handle.unlock(
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
    lock_ui.set_is_lock_activated(true);

    eprintln!("Ran till here");
    cast_spell(
        lock,
        None,
        None::<fn(Arc<RwLock<Box<dyn ForeignController>>>)>,
    )
}
