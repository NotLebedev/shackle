use std::fs;
use std::path::Path;

const STYLE_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/style");
const STYLE_MAIN: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/style/main.scss");

fn main() {
    let options = grass::Options::default()
        .load_path(STYLE_DIR)
        // GTK likes expanded style more.
        // Size is not that big of a deal, especially
        // since its compiles into the binary
        .style(grass::OutputStyle::Expanded);
    let css = grass::from_path(STYLE_MAIN, &options).expect("Failed to compile SCSS");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");
    fs::write(&dest_path, css).expect("Failed to write compiled CSS");

    println!("cargo:rerun-if-changed={}", STYLE_DIR);
}
