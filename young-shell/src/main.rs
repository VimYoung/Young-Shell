use std::{any::Any, cell::RefCell, env, error::Error, rc::Rc};

use slint::ComponentHandle;
use spell::{
    cast_spell,
    layer_properties::{DataType, ForeignController, LayerAnchor, LayerType, WindowConf},
    shared_context::SharedCore,
    slint_adapter::{SpellLayerShell, SpellSkiaWinAdapter},
    wayland_adapter::SpellWin,
};
slint::include_modules!();

impl ForeignController for State {
    fn get_type(&self, key: &str) -> DataType {
        match key {
            "is-power-menu-open" => DataType::Boolean(self.is_power_menu_open),
            _ => DataType::Panic,
        }
    }

    fn change_val(&mut self, key: &str, val: DataType) {
        match key {
            "is-power-menu-open" => {
                if let DataType::Boolean(value) = val {
                    self.is_power_menu_open = value;
                }
            }
            "string-type" => self.string_type = "hello".into(),
            "enumsss" => println!("{:?}", self.cards_type),
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // env::set_var("WAYLAND_DEBUG", "1");
    env::set_var("RUST_BACKTRACE", "1");
    // Dimentions for the widget size
    let width: u32 = 376; //1366;
    let height: u32 = 576; //768;
    let core = Rc::new(RefCell::new(SharedCore::new(width, height)));
    let window_adapter = SpellSkiaWinAdapter::new(core.clone(), width, height);
    let window_conf = WindowConf::new(
        width,
        height,
        (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
        (5, 0, 0, 10),
        LayerType::Top,
        core,
        window_adapter.clone(),
        false,
    );
    println!("Widnow Conf is set");

    let (waywindow, event_queue) = SpellWin::invoke_spell("counter-widget", window_conf);

    let _ = slint::platform::set_platform(Box::new(SpellLayerShell {
        window_adapter,
        time_since_start: std::time::Instant::now(),
    }));
    println!("platform for slint is set");
    let ui = Menu::new().unwrap();
    let state = Box::new(ui.get_state());
    println!("ui's new window created");

    let ui_handle = ui.as_weak().unwrap();
    //Slint Managing Inputs;
    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });
    //
    // println!("Casting the Spell");
    // ui.run()?;
    // Ok(())
    println!("spell is caseted");
    cast_spell(waywindow, event_queue, state, &mut |state_value| {
        let controller_val = state_value.read().unwrap();
        let inner = controller_val.as_ref();
        let val = inner.as_any().downcast_ref::<State>().unwrap().clone();
        ui_handle.set_state(val);
    })
}

// TODO the animations are jerky, you know the reason but you have to find a solution.
// TODO the cursor doesn't change from pointer to hand when clicking buttons, so the
// cursor needs to do that.
// TODO WlOutput is not properly implemented and managed. It is necessary for the proper
// functioning.
// TODO Lookup child creation in wayland, how can it be utilised.
// TODO Lookup popup in wayland to see if that helps in anything.
// TODO cursor shape management needs to be done.
// TODO there is some off by one error happeneing if either of width and height is not
// a multiple of 4.
