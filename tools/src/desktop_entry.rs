//! Generates the desktop entry for this application.

use clap::{Command, Arg, ArgMatches, ArgAction};
use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

fn main() {
    let matches = Command::new("desktop-entry-generate")
        .about("generates desktop entries")
        .arg(Arg::new("appid").long("appid").required(true))
        .arg(Arg::new("binary").long("binary").required(true))
        .arg(
            Arg::new("categories")
                .long("categories")
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(Arg::new("comment").long("comment").required(true))
        .arg(Arg::new("icon").long("icon").required(true))
        .arg(Arg::new("keywords").long("keywords").action(ArgAction::Append))
        .arg(Arg::new("name").long("name").required(true).required(true))
        .arg(Arg::new("prefix").long("prefix").required(true))
        .arg(Arg::new("startup-notify").long("startup-notify").action(ArgAction::SetTrue))
        .get_matches();

    write_desktop_entry(&matches).expect("failed to write desktop entry");
}

fn write_desktop_entry(matches: &ArgMatches) -> io::Result<()> {
    let appid = &*matches.get_one::<String>("appid").unwrap();
    let binary = &*matches.get_one::<String>("binary").unwrap();
    let categories = matches.get_many::<String>("categories").unwrap();
    let comment = &*matches.get_one::<String>("comment").unwrap();
    let icon = &*matches.get_one::<String>("icon").unwrap();
    let keywords = matches.get_many::<String>("keywords");
    let name = &*matches.get_one::<String>("name").unwrap();
    let prefix = &*matches.get_one::<String>("prefix").unwrap();

    let categories: Vec<&str> = categories.map(String::as_str).collect();
    let keywords: Option<Vec<&str>> = keywords.map(|keywords| keywords.map(String::as_str).collect());

    let exec_path = Path::new(&prefix).join("bin").join(&binary);
    let exec = exec_path.as_os_str().to_str().expect("prefix is not UTF-8");
    let mut desktop = File::create(["target/", appid, ".desktop"].concat().as_str())?;

    let entry = DesktopEntry::new(
        name,
        icon,
        DesktopType::Application({
            let mut app = Application::new(&categories, exec);

            if let Some(ref keywords) = keywords {
                app = app.keywords(keywords);
            }

            if matches.get_flag("startup-notify") {
                app = app.startup_notify();
            }

            app
        }),
    )
    .comment(comment);

    desktop.write_all(entry.to_string().as_bytes())
}
