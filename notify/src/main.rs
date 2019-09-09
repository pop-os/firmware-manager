use firmware_manager::{get_client, FirmwareSignal};
#[cfg(feature = "fwupd")]
use firmware_manager::{FwupdError, FwupdSignal};
use notify_rust::{Notification, Timeout};
use std::{
    path::Path,
    process::{exit, Command},
};

const UPDATES_FOUND: i32 = 3;

const GNOME_CONTROL_CENTER: &str = "/usr/share/applications/gnome-firmware-panel.desktop";

#[cfg(feature = "fwupd")]
use firmware_manager::{fwupd_scan, fwupd_updates, FwupdClient};

#[cfg(feature = "system76")]
use firmware_manager::{s76_firmware_is_active, s76_scan, System76Client};

fn main() {
    #[cfg(feature = "system76")]
    let s76 = get_client("system76", s76_firmware_is_active, System76Client::new);

    #[cfg(feature = "fwupd")]
    let fwupd = get_client::<_, _, FwupdError>(
        "fwupd",
        || true,
        || {
            let client = FwupdClient::new()?;
            client.ping()?;
            Ok(client)
        },
    );

    #[cfg(feature = "fwupd")]
    let http_client = &reqwest::Client::new();

    let event_handler = |event: FirmwareSignal| match event {
        #[cfg(feature = "fwupd")]
        FirmwareSignal::Fwupd(FwupdSignal { upgradeable, .. }) => {
            if upgradeable {
                notify();
            }
        }
        #[cfg(feature = "system76")]
        FirmwareSignal::S76System(info, ..) | FirmwareSignal::ThelioIo(info, ..) => {
            if info.latest.as_ref().map_or(false, |latest| latest.as_ref() != info.current.as_ref())
            {
                notify();
            }
        }
        _ => (),
    };

    #[cfg(feature = "system76")]
    {
        if let Some(ref client) = s76 {
            s76_scan(client, &event_handler);
        }
    }

    #[cfg(feature = "fwupd")]
    {
        if let Some(ref client) = fwupd {
            if let Err(why) = fwupd_updates(client, http_client) {
                eprintln!("failed to update fwupd remotes: {}", why);
            }

            fwupd_scan(client, &event_handler);
        }
    }
}

fn notify() {
    Notification::new()
        .summary("Firmware updates are available.")
        .body("Click here to install them.")
        .icon("firmware-manager")
        .appname("firmware-manager")
        .action("default", "default")
        // .hint(NotificationHint::Resident(true))
        .timeout(Timeout::Never)
        .show()
        .expect("failed to show desktop notification")
        .wait_for_action(|action| match action {
            "default" => {
                let (cmd, args): (&str, &[&str]) = if Path::new(GNOME_CONTROL_CENTER).exists() {
                    ("gnome-control-center", &["firmware"])
                } else {
                    ("com.system76.FirmwareManager", &[])
                };

                let _ = Command::new(cmd).args(args).status();
            }
            "__closed" => (),
            _ => (),
        });

    exit(UPDATES_FOUND);
}
