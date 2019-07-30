use std::{fs::File, env, io::Write};

markup::define! {
    Service<'a>(description: &'a str, appid: &'a str, exec: &'a str) {
        "[Unit]\n"
        "Description=" { markup::raw(description) } "\n"
        "Wants=" { markup::raw(appid) } ".timer\n"
        "\n"
        "[Service]\n"
        "ExecStart=" { markup::raw(exec) } "\n"
        "\n"
        "[Install]\n"
        "WantedBy=default.target\n"
    }
}

markup::define! {
    Timer<'a>(description: &'a str, appid: &'a str, minutes: u16) {
        "[Unit]\n"
        "Description=" { markup::raw(description) } "\n"
        "Requires=" { markup::raw(appid) } ".service\n"
        "\n"
        "[Timer]\n"
        "Unit=" { markup::raw(appid) } ".service\n"
        "OnUnitInactiveSec=" { markup::raw(minutes) } "m\n"
        "AccuracySec=1s\n"
        "\n"
        "[Install]\n"
        "WantedBy=timers.target\n"
    }
}

fn main() {
    let appid = env::var("APPID").unwrap();
    let prefix = env::var("prefix").unwrap();

    let timer_path = ["../target/", &appid, ".timer"].concat();
    let service_path = ["../target/", &appid, ".service"].concat();
    let exec = [&prefix, "/bin/", &appid].concat();

    let timer = Timer {
        description: "Checks for new firmware every day",
        appid: &appid,
        minutes: 1440
    };

    let service = Service {
        description: "Check for firmware updates, and display a notification if found",
        appid: &appid,
        exec: &exec
    };

    File::create(timer_path)
        .expect("failed to create timer service")
        .write_all(timer.to_string().as_bytes())
        .expect("failed to write timer service");

    File::create(service_path)
        .expect("failed to create service service")
        .write_all(service.to_string().as_bytes())
        .expect("failed to write service service");
}