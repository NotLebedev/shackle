pub mod fprint {
    /// net.reactivated.Fprint.Device
    pub mod device {
        include!(concat!(env!("OUT_DIR"), "/dbus-fprint-device.rs"));
    }

    /// net.reactivated.Fprint.Manager
    pub mod manager {
        include!(concat!(env!("OUT_DIR"), "/dbus-fprint-manager.rs"));
    }
}
