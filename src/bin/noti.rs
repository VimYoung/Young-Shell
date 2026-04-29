use std::{env, error::Error, thread};

use slint::{Color, ToSharedString};
use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
    vault::{NOTIFICATION_EVENT, NotificationManager, Timeout},
};
slint::include_modules!();
spell_framework::generate_widgets![YoungNC];

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::new(
        950,
        830,
        (Some(LayerAnchor::RIGHT), Some(LayerAnchor::BOTTOM)),
        (0, -250, 0, 0),
        LayerType::Top,
        BoardType::None,
        None,
    );

    let notinc = YoungNCSpell::invoke_spell("youngnc", window_conf);

    notinc.on_a_input_region({
        let handle = notinc.get_handler().clone();
        move |x, y, width, height| {
            handle.add_input_region(x, y, width, height);
        }
    });

    notinc.on_r_input_region({
        let handle = notinc.get_handler().clone();
        move |x, y, width, height| {
            handle.subtract_input_region(x, y, width, height);
        }
    });

    notinc.on_noti_close(move |id| {
        thread::spawn(move || {
            let _ = NOTIFICATION_EVENT
                .get()
                .unwrap()
                .call_close(id.try_into().unwrap());
        });
    });

    notinc.subtract_input_region(0, 0, 950, 830);
    cast_spell!(notification: notinc)
}

impl NotificationManager for YoungNC {
    fn new_notification(
        &self,
        notification: spell_framework::vault::Notification,
    ) -> Result<(), spell_framework::vault::NotiError> {
        println!("New Notification called: {:#?}", notification);
        self.invoke_add_notif(
            notification.id as i32,
            notification.appname.to_shared_string(),
            notification.summary.to_shared_string(),
            notification.body.to_shared_string(),
            give_timeout(notification.timeout),
            Color::from_rgb_u8(63, 185, 80),
        );
        self.invoke_a_input_region(650, 0, 299, 825);
        Ok(())
    }
}

fn give_timeout(timeout: Timeout) -> i32 {
    match timeout {
        Timeout::Default => 5,
        Timeout::Never => 10,
        Timeout::Milliseconds(val) => val,
    }
}
