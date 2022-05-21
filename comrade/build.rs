use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use built::write_built_file;

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn write_built_file_extras() -> io::Result<()> {
    let name = capitalize(env::var("CARGO_PKG_NAME").unwrap().as_str());

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("built.rs");
    let mut file = fs::OpenOptions::new().append(true).open(dest_path)?;

    writeln!(file, "#[doc=r#\"The name of the package for display.\"#]")?;
    writeln!(file, "#[allow(dead_code)]")?;
    writeln!(file, "pub const PKG_NAME_DISPLAY: &str = r\"{}\";", name)?;

    Ok(())
}

fn main() {
    write_built_file().expect("Failed to acquire build-time information");
    write_built_file_extras().expect("Failed to add our extra meta data");
}
