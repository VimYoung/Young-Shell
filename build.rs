fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let library_paths =
        std::collections::HashMap::from([("sleek".to_string(), manifest_dir.join("ui/sleek-ui/"))]);
    let config = slint_build::CompilerConfiguration::new()
        .with_style("cosmic-dark".into())
        .with_library_paths(library_paths);
    // let config = slint_build::CompilerConfiguration::new().with_style("material-dark".into());
    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}
