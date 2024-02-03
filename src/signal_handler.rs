use log::info;
use tokio::signal::unix::signal;

use crate::app::Message;

enum SignalResult {
    Recieved,
    Error,
}

impl SignalResult {
    fn recieved() -> Self {
        info!("Recieved SIGUSR1.");
        Self::Recieved
    }

    fn error() -> Self {
        info!("Failed to listen for SIGUSR1. Unlocking via signal is disabled");
        Self::Error
    }

    fn to_message(self) -> Message {
        match self {
            SignalResult::Recieved => Message::Unlock,
            SignalResult::Error => Message::Ignore,
        }
    }
}

async fn sighandler() -> SignalResult {
    let Ok(mut sigusr1) = signal(tokio::signal::unix::SignalKind::user_defined1()) else {
        return SignalResult::error();
    };

    match sigusr1.recv().await {
        Some(_) => SignalResult::recieved(),
        None => SignalResult::error(),
    }
}

pub fn signal_command() -> iced::Command<Message> {
    return iced::Command::perform(sighandler(), SignalResult::to_message);
}
