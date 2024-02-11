use log::info;
use tokio::signal::unix::signal;

use crate::app::Message;

pub async fn sighandler() -> Message {
    let Ok(mut sigusr1) = signal(tokio::signal::unix::SignalKind::user_defined1()) else {
        info!("Failed to listen for SIGUSR1. Unlocking via signal is disabled");
        return Message::Ignore;
    };

    // Interestingly enough the Option returned by recv() is
    // does not mean success/failure, but rather if the stream
    // was closed or not. But only one signal is recieved so
    // no need to care about this
    sigusr1.recv().await;
    info!("Recieved SIGUSR1.");
    Message::Unlock
}
