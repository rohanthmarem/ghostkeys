//! Manual real-editor check: cargo run --example background_probe -- <PID> [text]
//! Use a disposable editor in the named process. This sends actual background edits.
#[cfg(target_os = "macos")]
fn main() -> Result<(), String> {
    let pid = std::env::args()
        .nth(1)
        .ok_or("Pass the disposable editor's process ID")?
        .parse::<i32>()
        .map_err(|_| "Invalid process ID")?;
    println!("{}", ghostkeys_lib::background::capture_target(Some(pid))?);
    if let Some(text) = std::env::args().nth(2) {
        let mut writer =
            ghostkeys_lib::background::mac::BackgroundWriter::new()?.ok_or("No editor captured")?;
        for c in text.chars() {
            writer.type_char(c)?;
        }
        println!("{}", ghostkeys_lib::background::get_background_target());
        println!("Background edit completed and every character was observed in the editor.");
    }
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This manual check requires macOS.");
}
