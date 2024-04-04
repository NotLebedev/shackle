use std::{sync::Arc, time::Duration};

use dbus::{
    message::SignalArgs,
    nonblock::{self, MsgMatch, SyncConnection},
};
use dbus_tokio::connection::new_system_sync;
use fprint::{device::Device, manager::Manager};
use futures::select;
use iced::futures::StreamExt;
use log::{info, warn};

use crate::{
    app::Message,
    dbus::{fprint::device::DeviceVerifyStatus, login1::ManagerPrepareForSleep},
};

pub async fn fprint(await_wakeup: bool) -> Message {
    let Ok((resource, conn)) = new_system_sync() else {
        return Message::Ignore;
    };

    let _handle = tokio::spawn(async {
        let err = resource.await;
        info!("Lost connection to D-Bus: {err}");
    });

    if await_wakeup {
        wait_for_wakeup(conn.clone()).await;
    }

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

        let Ok(result) = attempt_verification(conn.clone()).await else {
            return Message::Ignore;
        };

        match result {
            VerifyResult::Match => {
                let _ = device.release().await;
                return Message::Unlock;
            }
            VerifyResult::NoMatch | VerifyResult::UnknownError | VerifyResult::UnexpectedWakeup => {
                if let Err(err) = device.verify_stop().await {
                    info!("Failed to stop verification: {err}");
                }
            }
            VerifyResult::Disconnected => {
                warn!("Fingerprint device disconnected");
                let _ = device.release().await;
                return Message::Ignore;
            }
            VerifyResult::Suspended => {
                info!("Device suspending. Pausing fingerprint verification.");
                if let Err(err) = device.verify_stop().await {
                    info!("Failed to stop verification: {err}");
                    // If device did not pause continue as normal
                    // It may have disconnected
                }
                wait_for_wakeup(conn.clone()).await;
            }
        }
    }
}

async fn add_match<T: SignalArgs>(connection: Arc<SyncConnection>) -> Result<MsgMatch, ()> {
    let handle = connection
        .clone()
        .add_match(T::match_rule(None, None).static_clone())
        .await
        .map_err(|_| ())?;

    Ok(handle)
}

/// This enum represents possible Verify Statuses that are
/// an end of single verification attempt
///
/// Per fprint dbus specification all other statuses mean
/// that verification is still ongoing
enum VerifyResult {
    /// The verification succeeded, Device. VerifyStop should now be called
    Match,
    /// The verification did not match, Device. VerifyStop should now be called
    NoMatch,
    /// The device was disconnected during the verification,
    /// no other actions should be taken, and you shouldn't use the device any more
    Disconnected,
    /// An unknown error occurred (usually a driver problem),
    /// Device. VerifyStop should now be called
    UnknownError,
    /// Verification was interrupted by device going to sleep. Wait
    /// until device signals wake before resuming verification
    Suspended,
    /// Verification was interrupted by device waking up. Time went device went to
    /// sleep was missed so it's best to restart verification process
    UnexpectedWakeup,
}

async fn attempt_verification(connection: Arc<SyncConnection>) -> Result<VerifyResult, ()> {
    let Ok(verify) = add_match::<DeviceVerifyStatus>(connection.clone()).await else {
        info!("Failed to start verification");
        return Err(());
    };
    let Ok(sleep) = add_match::<ManagerPrepareForSleep>(connection.clone()).await else {
        info!("Failed to wait for sleep");
        return Err(());
    };

    let (verify_match, mut verify) = verify.stream::<DeviceVerifyStatus>();
    let (sleep_match, mut sleep) = sleep.stream::<ManagerPrepareForSleep>();

    let result = loop {
        select! {
            status = verify.select_next_some() => {
                let result = status.1.result;
                info!("Verification status {result} recived.");
                match result.as_str() {
                    "verify-retry-scan"
                    | "verify-swipe-too-short"
                    | "verify-finger-not-centered"
                    | "verify-remove-and-retry" => (),
                    "verify-match" => break VerifyResult::Match,
                    "verify-no-match" => break VerifyResult::NoMatch,
                    "verify-disconnected" => break VerifyResult::Disconnected,
                    "verify-unknown-error" => break VerifyResult::UnknownError,
                    // Unexpected result from DeviceVerifyStatus
                    // Best not to do anything
                    _ => break VerifyResult::UnknownError,
                }
            }

            status = sleep.select_next_some() => {
                info!("Prepare for sleep: {}", status.1.start);
                if status.1.start {
                    break VerifyResult::Suspended;
                }
                else {
                    break VerifyResult::UnexpectedWakeup;
                }
            }

            complete => {
                info!("Stream ended");
                break VerifyResult::UnknownError;
            },
        }
    };
    // It's important to remove match before doing next round
    // as stated in docs https://docs.rs/dbus/0.9.7/dbus/nonblock/struct.MsgMatch.html
    // drop does not properly dispose of match (because drop can't be async)
    let _ = connection.remove_match(verify_match.token()).await;
    let _ = connection.remove_match(sleep_match.token()).await;
    return Ok(result);
}

/// Await until device wakes up and resume verification
async fn wait_for_wakeup<'a>(connection: Arc<SyncConnection>) {
    let Ok(awake) = add_match::<ManagerPrepareForSleep>(connection.clone()).await else {
        info!("Failed to start wait for sleep");
        // This is non-critical. Worst case fprint will wait a little and
        // throw an unknown error.
        return;
    };

    let (awake_match, mut awake) = awake.stream::<ManagerPrepareForSleep>();
    awake.next().await;
    info!("Device awaking. Resuming verification.");

    let _ = connection.remove_match(awake_match.token()).await;
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

mod login1 {
    include!(concat!(env!("OUT_DIR"), "/dbus-login1-manager.rs"));
}
