use std::{
    any::Any,
    env,
    error::Error,
    io::{self, BufRead, BufReader},
    os::unix::net::UnixStream,
    path::Path,
    process::Command,
    rc::Rc,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use spell_framework::{
    enchant_spells,
    forge::Forge,
    layer_properties::{
        BoardType, DataType, ForeignController, LayerAnchor, LayerType, WindowConf,
    },
    slint_adapter::SpellMultiWinHandler,
    vault::AppSelector,
};
slint::include_modules!();
use slint::{ComponentHandle, Image};


impl ForeignController for BarState {
    fn get_type(&self, key: &str) -> DataType {
        match key {
            "is-search-on" => DataType::Boolean(self.is_search_on),
            _ => DataType::Panic,
        }
    }

    fn change_val(&mut self, key: &str, val: DataType) {
        match key {
            "is-search-on" => {
                if let DataType::Boolean(value) = val {
                    self.is_search_on = value;
                }
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

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
    let mut windows = SpellMultiWinHandler::conjure_spells(vec![
        (
            "top-bar",
            WindowConf::new(
                1366,
                610,
                (Some(LayerAnchor::TOP), None),
                (0, 0, 0, 0),
                LayerType::Top,
                BoardType::None,
                true,
            ),
        ),
        (
            "menu",
            WindowConf::new(
                376,
                576,
                (Some(LayerAnchor::TOP), Some(LayerAnchor::LEFT)),
                (5, 0, 0, 10),
                LayerType::Top,
                BoardType::None,
                false,
            ),
        ),
        (
            "workspace",
            WindowConf::new(
                10,
                738,
                (Some(LayerAnchor::LEFT), None),
                (0, 0, 0, 0),
                LayerType::Top,
                BoardType::None,
                true,
            ),
        ),
    ]);
    let bar = TopBar::new().unwrap();
    let menu = Menu::new().unwrap();
    let workspace = Workspaces::new().unwrap();
    let [ref mut way_bar, ref mut way_menu, _] = windows[..] else {
        panic!("Error getting wayland handles");
    };

    let run_dir = env::var("XDG_RUNTIME_DIR");
    let inst_dir = env::var("HYPRLAND_INSTANCE_SIGNATURE");
    // let forge = Forge::new(way_bar.get_handler());

    if let Ok(run) = run_dir {
        if let Ok(inst) = inst_dir {
            let path = run + "/hypr/" + inst.as_str() + "/.socket2.sock";
            let unix_stream = UnixStream::connect(path).expect("Couldn't connect");
            unix_stream
                .set_nonblocking(true)
                .expect("couldn't set non blocking");

            let mut reader = BufReader::new(unix_stream);
            let workspace_n = workspace.as_weak().unwrap();
            workspace.on_refresh_workspaces(move|| {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Ok(_) => {
                        let curr_active = String::from_utf8( Command::new("sh").arg("-c")
                            .arg("hyprctl monitors -j | jq --argjson arg 0 '.[] | select(.id == 0).activeWorkspace.id'").output().expect("Couldn't run command").stdout).unwrap();
                        let filled = String::from_utf8(Command::new("sh")
                            .arg("-c")
                            .arg("hyprctl workspaces -j | jq '.[] | .id'")
                            .output().expect("couldn't run").stdout).unwrap();
                        let curr_active_num: i32 =  curr_active.trim().parse().unwrap();
                        if curr_active_num > 0 && curr_active_num < 11{
                            let mut v = vec![false; 10];
                            if (1..=10).contains(&curr_active_num) {
                                v[curr_active_num as usize - 1] = true;
                            }
                            workspace_n.set_is_active(Rc::new(slint::VecModel::from(v)).into());
                        }

                        let mut v = vec![false; 10];
                        let some_v: Vec<_> = filled.split('\n').collect();
                        some_v.iter().enumerate().for_each(|(i, m)|{
                            if i < (some_v.len() -1) && *m != "null" {
                                let m_int: i32 = m.trim().parse().unwrap();
                                if m_int > 0 && m_int < 11{
                                    v[ m_int as usize - 1] = true;
                                }
                            }
                        });

                        workspace_n.set_is_filled(Rc::new(slint::VecModel::from(v)).into());
                    }
                    Err(_) => todo!(),
                }
            });
        }
    }

    workspace.on_change_called(move |mut val| {
        val += 1;
        let val_str: String = val.to_string();
        let _ = Command::new("hyprctl")
            .arg("dispatch")
            .arg("workspace")
            .arg(&val_str)
            .output();
    });

    let bar_n = bar.as_weak().unwrap();
    // forge.add_event(Duration::from_secs(2), move |_| {
    //     let output = Command::new("date")
    //         .args(["+%I:%M"])
    //         .output()
    //         .expect("failed to execute process");
    //
    //     let am_pm = String::from_utf8(
    //         Command::new("date")
    //             .args(["+%p"])
    //             .output()
    //             .expect("couldn't run")
    //             .stdout,
    //     )
    //     .unwrap();
    //     let mut time = String::from_utf8(output.stdout).unwrap();
    //     time = format!("{} {}", time.trim(), am_pm.trim());
    //     // println!("/{}/", time);
    //     bar_n.set_time_var(time.into());
    // });
    //
    let bar_tx = way_bar.get_handler();
    let menu_tx = way_menu.get_handler();
    let state = Box::new(menu.get_state());
    let bar_state = Box::new(bar.get_state());
    let menu_handle = menu.as_weak().unwrap();
    let bar_handle = bar.as_weak().unwrap();

    let app_selector = AppSelector::default();
    let app_data_slint: Vec<AppLineData> = app_selector
        .get_primary()
        .map(|value| {
            let imag_path_val: String;
            if let Some(val) = value.image_path.clone() {
                imag_path_val = val;
            } else {
                imag_path_val = "/home/ramayen/assets/kitty.png".to_string();
            }
            AppLineData {
                image: Image::load_from_path(Path::new(&imag_path_val))
                    .expect("Error loading image"),
                name: value.name.clone().into(),
                action: value
                    .exec_comm
                    .clone()
                    .unwrap_or_else(|| "no comm".to_string())
                    .into(),
            }
        })
        .collect();
    let vac_model = Rc::new(slint::VecModel::from(app_data_slint));
    bar.set_app_lines(vac_model.clone().into());
    bar.on_open_app(|string_val| {
        let command_val: &str;
        let mut args_vec: Vec<&str> = Vec::new();
        let binding = string_val.to_string();
        if let Some((command, args)) = binding.split_once(' ') {
            command_val = command;
            args_vec = args.split(' ').collect();
        } else {
            command_val = &string_val;
        };
        let mut final_comm = Command::new(command_val);
        if !args_vec.is_empty() {
            args_vec.iter().for_each(|argument| {
                final_comm.arg(argument);
            });
        }
        thread::spawn(move || {
            final_comm.output().unwrap();
        });
        println!("{string_val:?}");
    });

    let bar_tx_another = bar_tx.clone();
    let bar_tx_clone = bar_tx.clone();
    bar.on_request_menu_toggle({
        let bar_handle = bar.as_weak().unwrap();
        move || {
            if bar_handle.get_state().is_search_on {
                bar_handle.set_state(BarState {is_search_on: true});
                bar_tx_another.remove_focus();
                bar_tx_clone.subtract_input_region(0, 35, 1366, 576);
            } else {
                bar_handle.set_state(BarState {is_search_on: false});
                bar_tx_another.grab_focus();
                bar_tx_clone.add_input_region(0, 35, 1366, 576);
            }
        }
    });
    let bar_tx_clone_a = bar_tx.clone();
    let bar_tx_clone_b = bar_tx.clone();
    bar_tx_clone_b.subtract_input_region(0, 35, 1366, 576);
    bar.on_walls_window_called({
        let bar_handle = bar.as_weak().unwrap();
        move || {
            if !bar_handle.get_walls_open() {
                bar_tx_clone_a.add_input_region(0, 35, 1366, 315);
            } else {
                bar_tx_clone_a.subtract_input_region(0, 35, 1366, 315);
            }
        }
    });

    bar.on_query_applications({
        let bar_handle = bar.as_weak().unwrap();
        move |query_value| {
            let app_data_native = app_selector.query_primary(query_value.as_ref(), 15);
            // println!("{:#?}", app_data_native);
            let app_data_slint: Vec<AppLineData> = app_data_native
                .iter()
                .map(|value| {
                    let imag_path_val: String;
                    if let Some(val) = value.image_path.clone() {
                        imag_path_val = val;
                    } else {
                        imag_path_val = "/home/ramayen/assets/kitty.png".to_string();
                    }
                    AppLineData {
                        image: Image::load_from_path(Path::new(&imag_path_val))
                            .expect("Error loading image"),
                        name: value.name.clone().into(),
                        action: value
                            .exec_comm
                            .clone()
                            .unwrap_or_else(|| "no comm".to_string())
                            .into(),
                    }
                })
                .collect();
            let vac_model = Rc::new(slint::VecModel::from(app_data_slint));
            bar_handle.set_app_lines(vac_model.clone().into());
        }
    });
    menu_tx.toggle();
    // bar.global::<Rice>().on_get_volume(|| {
    //     let val = Command::new("sh").arg("-c").arg("pactl list sinks | grep '^[[:space:]]Volume:' | head -n $(( $SINK + 1 )) | tail -n 1 | sed -e 's,.* \\([0-9][0-9]*\\)%.*,\\1,'").output().unwrap();
    //     let output_str =  String::from_utf8(val.stderr).unwrap();
    //     println!("{}",output_str);
    //     output_str.into()
    // });
    //
    // let m = bar.as_weak().clone();
    // forge.add_event(Duration::from_secs(1), |_| {
    //     // bar.global::<Rice>().invoke_get_volume();
    //     m._walls_window_called();
    // });
    enchant_spells::<Box<dyn FnMut(Arc<RwLock<Box<dyn ForeignController>>>)>>(
        windows,
        vec![Some(Arc::new(RwLock::new(bar_state))), Some(Arc::new(RwLock::new(state))), None],
        vec![
            Some(Box::new(
                move |state_value: Arc<RwLock<Box<dyn ForeignController + 'static>>>| {
                    println!("Entered in the callback");
                    let controller_val = state_value.read().unwrap();
                    let inner = controller_val.as_ref();
                    let val = inner.as_any().downcast_ref::<BarState>().unwrap();
                    bar_handle.set_state(val.clone());
                },
            )),
            Some(Box::new(
                move |state_value: Arc<RwLock<Box<dyn ForeignController + 'static>>>| {
                    println!("Entered in the callback");
                    let controller_val = state_value.read().unwrap();
                    let inner = controller_val.as_ref();
                    let val = inner.as_any().downcast_ref::<State>().unwrap();
                    menu_handle.set_state(val.clone());
                },
            )),
            None,
        ],
    )
}

// TODO the cursor doesn't change from pointer to hand when clicking buttons, so the
// cursor needs to do that.
// TODO Lookup child creation in wayland, how can it be utilised.
// TODO Lookup popup in wayland to see if that helps in anything.
