//! Generates the desktop entry for this application.

use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn main() {
    let appid = env::var("APPID").expect("no appid env var");
    let prefix = env::var("prefix").expect("missing prefix env var");
    write_desktop_entry(&prefix, &appid, &appid, desktop_entry)
        .expect("failed to write desktop entry");
}

fn write_desktop_entry<F: FnOnce(&str) -> DesktopEntry>(
    prefix: &str,
    name: &str,
    appid: &str,
    func: F
) -> io::Result<()> {
    let exec_path = Path::new(&prefix).join("bin").join(name);
    let exec = exec_path.as_os_str().to_str().expect("prefix is not UTF-8");
    let mut desktop = File::create(["target/", appid, ".desktop"].concat().as_str())?;
    desktop.write_all(func(exec).to_string().as_bytes())
}

fn desktop_entry(exec: &str) -> DesktopEntry {
    DesktopEntry::new(
        "Firmware Manager",
        "firmware-manager",
        DesktopType::Application(
            Application::new(&["System", "GTK"], exec)
                .keywords(&["firmware", "system76", "fwupd", "lvfs"])
                .startup_notify(),
        ),
    )
    .comment("Manage system and device firmware")
}
