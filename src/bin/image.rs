use slint::{ComponentHandle, Image, SharedString};
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let im = ImageEx::new().unwrap();
    im.run()?;
}
