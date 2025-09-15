use std::{
    env,
    error::Error,
    io::{self, BufRead, BufReader},
    os::unix::net::UnixStream,
    process::Command,
    rc::Rc,
    sync::{Arc, RwLock},
};

use slint::ComponentHandle;
use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, ForeignController, LayerAnchor, LayerType, WindowConf},
    wayland_adapter::SpellWin,
};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::new(
        7,
        768,
        (Some(LayerAnchor::LEFT), Some(LayerAnchor::TOP)),
        (0, 0, 0, 0),
        LayerType::Top,
        BoardType::None,
        true,
    );

    let way_work = SpellWin::invoke_spell("workspace", window_conf);
    let workspace = Workspaces::new().unwrap();

    let run_dir = env::var("XDG_RUNTIME_DIR");
    let inst_dir = env::var("HYPRLAND_INSTANCE_SIGNATURE");

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
                            // println!("/{m}/");
                            // println!("{}, //{:?}", some_v.len(), some_v);
                            if i < (some_v.len() -1) && *m != "null" {
                                let m_int: i32 = m.trim().parse().unwrap();
                                if m_int > 0 && m_int < 11{
                                    v[ m_int as usize - 1] = true;
                                }
                            }
                        });

                        workspace_n.set_is_filled(Rc::new(slint::VecModel::from(v)).into());
                        // println!("{}, /{}", curr_active, filled);
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
    cast_spell(
        way_work,
        None,
        None::<fn(Arc<RwLock<Box<dyn ForeignController>>>)>,
    )
}
