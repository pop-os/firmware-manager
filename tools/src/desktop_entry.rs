//! Generates the desktop entry for this application.

use std::{
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

const NAME: &str = "Firmware Manager";
const APPID: &str = "com.system76.FirmwareManager";
const ICON: &str = "firmware-manager";
const CATEGORIES: &str = "System";
const KEYWORDS: &[&str] = &["firmware", "system76", "fwupd", "lvfs"];

fn main() {
    let path = prefix_path();
    let path = path.as_os_str().to_str().expect("prefix is not UTF-8");

    let mut desktop = File::create(["target/", APPID, ".desktop"].concat().as_str())
        .expect("failed to create desktop entry file");
    desktop_entry(&mut desktop, NAME, ICON, CATEGORIES, KEYWORDS, path);
}

/// Fetch the prefix path from the environment, or use `/usr/local` by default.
fn prefix_path() -> PathBuf {
    let mut _tmp: String;
    let prefix = match env::var("prefix") {
        Ok(var) => {
            _tmp = var;
            &_tmp
        }
        _ => "/usr/local",
    };

    Path::new(prefix).join("bin").join(APPID)
}

/// Generate the desktop entry based on the template in the assets directory.
fn desktop_entry(
    out: &mut File,
    name: &str,
    icon: &str,
    categories: &str,
    keywords: &[&str],
    path: &str,
) {
    fn format_list(list: &[&str]) -> Box<str> {
        let mut formatted =
            String::with_capacity(list.iter().map(|s| s.len()).sum::<usize>() + list.len());
        for item in list.iter() {
            formatted.push_str(item);
            formatted.push(';');
        }

        formatted.into()
    }

    let _ = writeln!(
        out,
        include_str!("template.desktop"),
        name,
        icon,
        categories,
        format_list(keywords),
        path
    );
}
