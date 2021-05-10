use fomat_macros::fomat;
use std::{env, fs::File, io::Write};

fn service(description: &str, appid: &str, exec: &str) -> String {
    fomat!(
        "[Unit]\n"
        "Description=" (description) "\n"
        "Wants=" (appid) ".timer\n"
        "\n"
        "[Service]\n"
        "ExecStart=" (exec) "\n"
        "\n"
        "[Install]\n"
        "WantedBy=default.target\n"
    )
}

fn timer(description: &str, appid: &str, minutes: u16) -> String {
    fomat!(
        "[Unit]\n"
        "Description=" (description) "\n"
        "Requires=" (appid) ".service\n"
        "\n"
        "[Timer]\n"
        "Unit=" (appid) ".service\n"
        "OnUnitInactiveSec=" (minutes) "m\n"
        "AccuracySec=1s\n"
        "\n"
        "[Install]\n"
        "WantedBy=timers.target\n"
    )
}

fn main() {
    let appid = env::var("APPID").unwrap();
    let prefix = env::var("prefix").unwrap();

    let timer_path = ["../target/", &appid, ".timer"].concat();
    let service_path = ["../target/", &appid, ".service"].concat();
    let exec = [&prefix, "/bin/", &appid].concat();

    let timer = timer("Checks for new firmware every day", &appid, 1440);

    let service =
        service("Check for firmware updates, and display a notification if found", &appid, &exec);

    File::create(timer_path)
        .expect("failed to create timer service")
        .write_all(timer.to_string().as_bytes())
        .expect("failed to write timer service");

    File::create(service_path)
        .expect("failed to create service service")
        .write_all(service.to_string().as_bytes())
        .expect("failed to write service service");
}
