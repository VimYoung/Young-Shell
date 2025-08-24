use std::{env, error::Error, time::Duration};

use slint::ComponentHandle;
use spell_framework::{
    layer_properties::{TimeoutAction, Timer},
    wayland_adapter::{run_lock, SpellLock, SpellSlintLock},
};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let (mut lock, event_loop, event_queue) = SpellLock::invoke_lock_spell();
    let lock_ui = LockScreen::new().unwrap();
    let looop_handle = event_loop.handle().clone();
    lock_ui.on_check_pass({
        let lock_handle = lock_ui.as_weak();
        move |string_val| {
            // let lock_handle_a = lock_handle.clone();
            let lock_handle_a = lock_handle.clone().unwrap();
            looop_handle
                .insert_source(
                    Timer::from_duration(Duration::from_secs(5)),
                    move |_, _, app_data| {
                        if app_data.unlock(None, string_val.as_str()).is_err() {
                            lock_handle_a.set_lock_error(true);
                        }
                        TimeoutAction::Drop
                    },
                )
                .unwrap();
        }
    });

    eprintln!("Ran till here");
    run_lock(lock, event_loop, event_queue)
    // cast_spell::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(waywindow, None, None)
}
