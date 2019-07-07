//! Generates the desktop entry for this application.

use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

const APPID: &str = "com.system76.FirmwareManager";

fn main() {
    let prefix = env::var("prefix").expect("missing prefix env var");
    let exec_path = Path::new(&prefix).join("bin").join(APPID);
    let exec = exec_path.as_os_str().to_str().expect("prefix is not UTF-8");

    let mut desktop = File::create(["target/", APPID, ".desktop"].concat().as_str())
        .expect("failed to create desktop entry file");

    let entry = DesktopEntry::new(
        "Firmware Manager",
        "firmware-manager",
        DesktopType::Application(
            Application::new(&["System", "GTK"], exec)
                .keywords(&["firmware", "system76", "fwupd", "lvfs"])
                .startup_notify(),
        ),
    )
    .comment("Mange system and device firmware");

    desktop.write_all(entry.to_string().as_bytes());
}
