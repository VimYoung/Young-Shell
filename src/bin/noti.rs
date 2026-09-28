use std::{env, error::Error, rc::Rc, thread};

use slint::{Color, ToSharedString, VecModel};
use spell_framework::{
    cast_spell,
    layer_properties::{LayerAnchor, LayerType, WindowConf},
    vault::{NOTIFICATION_EVENT, NotificationManager, Timeout},
};
slint::include_modules!();
spell_framework::generate_widgets![YoungNC];

fn main() -> Result<(), Box<dyn Error>> {
    let notinc = YoungNCSpell::invoke_spell(
        "youngnc",
        WindowConf::builder()
            .width(950_u32)
            .height(830_u32)
            .anchor_1(LayerAnchor::RIGHT)
            .anchor_2(LayerAnchor::BOTTOM)
            .margins(0, -250, 0, 0)
            .layer_type(LayerType::Top)
            .build()
            .unwrap(),
    );

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
        // FIXME: Very poor design
        thread::spawn(move || {
            let _ = NOTIFICATION_EVENT.get().unwrap().call_close(
                id.try_into().unwrap(),
                spell_framework::vault::CloseReason::Dismissed,
            );
        });
    });

    notinc.on_action_called(|action, id| {
        // FIXME: Very poor design
        thread::spawn(move || {
            let _ = NOTIFICATION_EVENT
                .get()
                .unwrap()
                .action_invoked(id as u32, action.as_str());
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
        let actions: Vec<NotiAction> = notification
            .actions
            .chunks_exact(2)
            .map(|val| NotiAction {
                action: val[0].to_shared_string(), //SharedString::from(val[0]),
                display: val[1].to_shared_string(),
            })
            .collect();
        self.invoke_add_notif(
            notification.id as i32,
            notification.appname.to_shared_string(),
            notification.summary.to_shared_string(),
            notification.body.to_shared_string(),
            give_timeout(notification.timeout),
            Color::from_rgb_u8(63, 185, 80),
            Rc::new(VecModel::from(actions)).into(),
        );
        Ok(())
    }

    fn close_notification(&self, _id: u32) -> Result<(), spell_framework::vault::NotiError> {
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
