use std::{sync::Arc, time::Duration};

use dbus::{
    channel::Token,
    message::SignalArgs,
    nonblock::{self, MsgMatch, SyncConnection},
};
use dbus_tokio::connection::new_system_sync;
use fprint::{device::Device, manager::Manager};
use iced::futures::StreamExt;
use log::{info, warn};

use crate::{app::Message, dbus::fprint::device::DeviceVerifyStatus};

pub async fn fprint() -> Message {
    let Ok((resource, conn)) = new_system_sync() else {
        return Message::Ignore;
    };

    let _handle = tokio::spawn(async {
        let err = resource.await;
        info!("Lost connection to D-Bus: {err}");
    });

    let fprint_manager = nonblock::Proxy::new(
        "net.reactivated.Fprint",
        "/net/reactivated/Fprint/Manager",
        Duration::from_secs(2),
        conn.clone(),
    );

    let Ok(dev) = fprint_manager.get_default_device().await else {
        info!("No default fingerprint device. Check if fprintd-tod is installed.");
        return Message::Ignore;
    };

    info!("Default device: {dev:?}");

    let device = nonblock::Proxy::new(
        "net.reactivated.Fprint",
        dev,
        Duration::from_secs(2),
        conn.clone(),
    );

    // According to fprint dbus specification empty string means current user
    // The documentation advises to use this option over explicit username
    if let Err(err) = device.claim("").await {
        info!("Failed to claim device: {err}");
        return Message::Ignore;
    };

    info!("Claimed fingerprint device. Starting verification");

    loop {
        if let Err(err) = device.verify_start("any").await {
            info!("Failed to start verification: {err}");
            return Message::Ignore;
        };

        let Ok(verification) = add_match::<DeviceVerifyStatus>(conn.clone()).await else {
            info!("Failed to start verification");
            return Message::Ignore;
        };

        let result = communicate(verification.handle).await;

        // It's important to remove match before doing next round
        // as stated in docs https://docs.rs/dbus/0.9.7/dbus/nonblock/struct.MsgMatch.html
        // drop does not properly dispose of match (because drop can't be async)
        let _ = conn.remove_match(verification.token).await;

        match result {
            VerifyResult::Match => {
                let _ = device.release().await;
                return Message::Unlock;
            }
            VerifyResult::NoMatch => {
                if let Err(err) = device.verify_stop().await {
                    info!("Failed to stop verification: {err}");
                }
            }
            VerifyResult::UnknownError => {
                if let Err(err) = device.verify_stop().await {
                    info!("Failed to stop verification: {err}");
                }
            }
            VerifyResult::Disconnected => {
                warn!("Fingerprint device disconnected");
                let _ = device.release().await;
                return Message::Ignore;
            }
        }
    }
}

struct Match {
    handle: MsgMatch,
    token: Token,
}
async fn add_match<T: SignalArgs>(connection: Arc<SyncConnection>) -> Result<Match, ()> {
    let handle = connection
        .clone()
        .add_match(T::match_rule(None, None).static_clone())
        .await
        .map_err(|_| ())?;

    let token = handle.token();
    Ok(Match { handle, token })
}

/// This enum represents possible Verify Statuses that are
/// an end of single verification attempt
/// Per fprint dbus specification all other statuses mean
/// that verification is still ongoing
enum VerifyResult {
    /// The verification succeeded, Device.VerifyStop should now be called
    Match,
    /// The verification did not match, Device.VerifyStop should now be called
    NoMatch,
    /// The device was disconnected during the verification,
    /// no other actions should be taken, and you shouldn't use the device any more
    Disconnected,
    /// An unknown error occurred (usually a driver problem),
    /// Device.VerifyStop should now be called.
    UnknownError,
}

async fn communicate(handle: MsgMatch) -> VerifyResult {
    let (mut item, mut rest) = handle.stream::<DeviceVerifyStatus>().1.into_future().await;

    loop {
        if let Some((_, DeviceVerifyStatus { result, .. })) = item {
            info!("Verification status {result} recived.");
            match result.as_str() {
                "verify-match" => return VerifyResult::Match,
                "verify-no-match" => return VerifyResult::NoMatch,
                "verify-disconnected" => return VerifyResult::Disconnected,
                "verify-unknown-error" => return VerifyResult::UnknownError,
                "verify-retry-scan"
                | "verify-swipe-too-short"
                | "verify-finger-not-centered"
                | "verify-remove-and-retry" => {
                    // These statuses mean that verification is still ongoing
                    // More messages are to be expected
                    (item, rest) = rest.into_future().await;
                }
                // Unexpected result from DeviceVerifyStatus
                // Best not to do anything
                _ => return VerifyResult::UnknownError,
            }
        } else {
            // Stream ended without successfull verification
            // This is unexpected, so best to assume that dbus handles
            // everything by itself
            return VerifyResult::UnknownError;
        }
    }
}

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
