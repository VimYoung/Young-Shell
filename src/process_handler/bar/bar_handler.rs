use battery::{Manager, State};
use slint::Weak;

use crate::{BatteryData, BatteryState, TopBar, TopBarSpell, process_handler::bar::BarMessage};

pub struct BarHandler {
    bar_weak: Weak<TopBar>,
    battery_manager: Manager,
}

impl BarHandler {
    pub fn new(bar_weak: Weak<TopBar>) -> Self {
        BarHandler {
            bar_weak,
            battery_manager: Manager::new().unwrap(),
        }
    }

    pub fn get_bar_instance_weak(&self) -> Weak<TopBar> {
        self.bar_weak.clone()
    }

    pub fn process_message(&self, msg: BarMessage) {
        match msg {
            BarMessage::UpdateBattery => {
                if let Ok(mut batteries) = self.battery_manager.batteries()
                    && let Some(battery) = batteries.find_map(|x| x.ok())
                {
                    // FIXME: Only sets the last battery, doesn't manage
                    // multiple batteries.
                    let battery_percent: i32 = (battery.state_of_charge().value * 100.0) as i32;
                    let bar_copy = self.bar_weak.clone();
                    slint::invoke_from_event_loop(move || {
                        bar_copy.unwrap().set_battery_val(BatteryData {
                            value: battery_percent,
                            state: match battery.state() {
                                State::Full => BatteryState::Full,
                                State::Charging => BatteryState::Charging,
                                _ => BatteryState::Discharging,
                            },
                        });
                    })
                    .expect("Faild to send battery status");
                }
            }
        }
    }
}
