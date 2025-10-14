use std::process::Command;

fn main() {
    let workspaces: String = String::from_utf8(
        Command::new("niri")
            .arg("msg")
            .arg("workspaces")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let mut values = workspaces.split('\n');
    values.next_back();
    for val in values.skip(1) {
        println!("`{:?}`", val.trim().as_bytes());
        println!("`{:?}`", val);
    }
}
