mod fprint {
    /// net.reactivated.Fprint.Device
    pub mod device {
        include!(concat!(env!("OUT_DIR"), "/dbus-fprint-device.rs"));
    }

    /// net.reactivated.Fprint.Manager
    pub mod manager {
        include!(concat!(env!("OUT_DIR"), "/dbus-fprint-manager.rs"));
    }
}

use std::time::Duration;

use dbus::nonblock;
use dbus_tokio::connection::new_system_sync;
use fprint::{device::Device, manager::Manager};
use log::info;

use crate::app::Message;

pub async fn fprint() -> Message {
    let Ok((resource, conn)) = new_system_sync() else {
        return Message::Ignore;
    };

    let _handle = tokio::spawn(async {
        let err = resource.await;
        info!("Lost connection to D-Bus: {err}");
    });

    let manager = nonblock::Proxy::new(
        "net.reactivated.Fprint",
        "/net/reactivated/Fprint/Manager",
        Duration::from_secs(2),
        conn.clone(),
    );

    let Ok(dev) = manager.get_default_device().await else {
        info!("No default fingerprint device. Check if fprintd-tod is installed.");
        return Message::Ignore;
    };

    info!("Default device: {dev:?}");

    let device = nonblock::Proxy::new("net.reactivated.Fprint", dev, Duration::from_secs(2), conn);

    let Ok(fingers) = device.verify_start("any").await else {
        info!("No fingers enrolled. Use ");
        return Message::Ignore;
    };

    info!("Enrolled fingers {fingers:?}");

    Message::Ignore
}
