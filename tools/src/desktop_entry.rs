//! Generates the desktop entry for this application.

use clap::{App, Arg, ArgMatches};
use freedesktop_desktop_entry::{Application, DesktopEntry, DesktopType};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

fn main() {
    let matches = App::new("desktop-entry-generate")
        .about("generates desktop entries")
        .arg(Arg::with_name("appid").long("appid").takes_value(true).required(true))
        .arg(Arg::with_name("binary").long("binary").takes_value(true).required(true))
        .arg(
            Arg::with_name("categories")
                .long("categories")
                .multiple(true)
                .number_of_values(1)
                .required(true),
        )
        .arg(Arg::with_name("comment").long("comment").takes_value(true).required(true))
        .arg(Arg::with_name("icon").long("icon").takes_value(true).required(true))
        .arg(Arg::with_name("keywords").long("keywords").multiple(true).number_of_values(1))
        .arg(Arg::with_name("name").long("name").takes_value(true).required(true).required(true))
        .arg(Arg::with_name("prefix").long("prefix").takes_value(true).required(true))
        .arg(Arg::with_name("startup-notify").long("startup-notify"))
        .get_matches();

    write_desktop_entry(&matches).expect("failed to write desktop entry");
}

fn write_desktop_entry(matches: &ArgMatches) -> io::Result<()> {
    let appid = matches.value_of("appid").unwrap();
    let binary = matches.value_of("binary").unwrap();
    let categories = matches.values_of("categories").unwrap();
    let comment = matches.value_of("comment").unwrap();
    let icon = matches.value_of("icon").unwrap();
    let keywords = matches.values_of("keywords");
    let name = matches.value_of("name").unwrap();
    let prefix = matches.value_of("prefix").unwrap();

    let categories: Vec<&str> = categories.collect();
    let keywords: Option<Vec<&str>> = keywords.map(|keywords| keywords.collect());

    let exec_path = Path::new(&prefix).join("bin").join(&binary);
    let exec = exec_path.as_os_str().to_str().expect("prefix is not UTF-8");
    let mut desktop = File::create(["target/", &appid, ".desktop"].concat().as_str())?;

    let entry = DesktopEntry::new(
        &name,
        &icon,
        DesktopType::Application({
            let mut app = Application::new(&categories, exec);

            if let Some(ref keywords) = keywords {
                app = app.keywords(keywords);
            }

            if matches.is_present("startup-notify") {
                app = app.startup_notify();
            }

            app
        }),
    )
    .comment(&comment);

    desktop.write_all(entry.to_string().as_bytes())
}
