use futures::{select, StreamExt};
use log::{error, info, warn};
use zbus::{proxy, zvariant::OwnedObjectPath};

pub async fn check_fingerprint(await_wakeup: bool) -> bool {
    let Ok(connection) = zbus::Connection::system().await else {
        error!("Failed to connect to system bus.");
        return false;
    };

    let Ok(login_manager) = Login1ManagerProxy::new(&connection).await else {
        error!("Failed to connect to login1 manager.");
        return false;
    };

    if await_wakeup {
        wait_for_wakeup(login_manager.clone()).await;
    }

    let Ok(fprint_manager) = FprintManagerProxy::new(&connection).await else {
        error!("Failed to connect to login1 manager.");
        return false;
    };

    let Ok(device_path) = fprint_manager.get_default_device().await else {
        error!("No default fingerprint device. Check if fprintd-tod is installed.");
        return false;
    };

    info!("Default device: {device_path:?}");

    let Ok(device) = connect_to_device(&connection, device_path).await else {
        error!("Failed to connect to default device.");
        return false;
    };

    // According to fprint dbus specification empty string means current user
    // The documentation advises to use this option over explicit username
    if let Err(err) = device.claim("").await {
        info!("Failed to claim device: {err}");
        return false;
    };

    info!("Claimed fingerprint device. Starting verification");
    loop {
        if let Err(err) = device.verify_start("any").await {
            info!("Failed to start verification: {err}");
            return false;
        };

        let Ok(result) = attempt_verification(login_manager.clone(), device.clone()).await else {
            return false;
        };

        match result {
            VerifyResult::Match => {
                let _ = device.release().await;
                return true;
            }
            VerifyResult::NoMatch | VerifyResult::UnknownError | VerifyResult::UnexpectedWakeup => {
                if let Err(err) = device.verify_stop().await {
                    info!("Failed to stop verification: {err}");
                }
            }
            VerifyResult::Disconnected => {
                warn!("Fingerprint device disconnected");
                let _ = device.release().await;
                return false;
            }
            VerifyResult::Suspended => {
                info!("Device suspending. Pausing fingerprint verification.");
                if let Err(err) = device.verify_stop().await {
                    info!("Failed to stop verification: {err}");
                    // If device did not stop continue as normal
                    // It may have disconnected
                }
                wait_for_wakeup(login_manager.clone()).await;
            }
        }
    }
}

async fn connect_to_device<'a>(
    connection: &zbus::Connection,
    device: OwnedObjectPath,
) -> Result<FprintDeviceProxy<'a>, ()> {
    FprintDeviceProxy::builder(connection)
        .path(device)
        .map_err(|_| ())?
        .build()
        .await
        .map_err(|_| ())
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
    /// Verification was interrupted by device waking up. Time when device went to
    /// sleep was missed so it's best to restart verification process
    UnexpectedWakeup,
}

async fn attempt_verification(
    login1_manager: Login1ManagerProxy<'_>,
    device: FprintDeviceProxy<'_>,
) -> Result<VerifyResult, ()> {
    let Ok(mut verify) = device.receive_verify_status().await else {
        error!("Failed to start verification");
        return Err(());
    };
    let Ok(mut sleep) = login1_manager.receive_prepare_for_sleep().await else {
        error!("Failed to wait for sleep");
        return Err(());
    };

    let result = loop {
        select! {
            status = verify.select_next_some() => {
                let Ok(status) = status.args() else {
                    warn!("Failed to parse verify status.");
                    break VerifyResult::UnknownError;
                };

                info!("Verification status {} recived.", status.result);
                match status.result {
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
                let Ok(status) = status.args() else {
                    warn!("Failed to parse prepare for sleep.");
                    break VerifyResult::UnknownError;
                };

                info!("Prepare for sleep: {}", status.start);
                if status.start {
                    break VerifyResult::Suspended;
                }
                else {
                    break VerifyResult::UnexpectedWakeup;
                }
            }

            complete => {
                warn!("Failed to listen for dbus events.");
                break VerifyResult::UnknownError;
            },
        }
    };

    return Ok(result);
}

/// Await until device wakes up and resume verification
async fn wait_for_wakeup<'a>(login1_manager: Login1ManagerProxy<'a>) {
    let Ok(mut prepare_for_sleep_stream) = login1_manager.receive_prepare_for_sleep().await else {
        info!("Failed to start wait for sleep");
        // All errors here are uncritical. Worst case fprint will
        // wait a little and throw an unknown error.
        return;
    };

    info!("Waiting for wakeup.");

    while let Some(msg) = prepare_for_sleep_stream.next().await {
        let Ok(args) = msg.args() else {
            info!("Failed to parse prepare for sleep message. Check dbus settings.");
            return;
        };

        // We need to skip one or zero messages
        // with start == true until we get
        // start == false, which means sleep ended
        if !args.start {
            info!("Device awaking. Resuming verification.");
            break;
        }
    }
}

// To generate dbus proxy declarations use
// `zbus-xmlgen file your-dbus-here.xml` and copy needed
// method from there. zbus docs advice against using it
// as codegen tool and insist on manually working with
// generated bindings.
// https://dbus2.github.io/zbus/client.html#generating-the-trait-from-an-xml-interface

#[proxy(
    interface = "net.reactivated.Fprint.Manager",
    default_path = "/net/reactivated/Fprint/Manager",
    default_service = "net.reactivated.Fprint"
)]
pub trait FprintManager {
    fn get_default_device(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "net.reactivated.Fprint.Device",
    default_service = "net.reactivated.Fprint",
    assume_defaults = true
)]
pub trait FprintDevice {
    fn claim(&self, username: &str) -> zbus::Result<()>;
    fn release(&self) -> zbus::Result<()>;

    fn verify_start(&self, finger_name: &str) -> zbus::Result<()>;
    fn verify_stop(&self) -> zbus::Result<()>;

    #[zbus(signal)]
    fn verify_status(&self, result: &str, done: bool) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_path = "/org/freedesktop/login1",
    default_service = "org.freedesktop.login1",
    assume_defaults = true
)]
pub trait Login1Manager {
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;
}
