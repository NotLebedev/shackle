use log::info;

use crate::app::Message;

pub async fn check_password(password: String) -> Message {
    let Ok(mut client) = pam::Client::with_password("shackle") else {
        info!("Failed to initialize PAM client. Session won't be unlocked.");
        return Message::Ignore;
    };

    let Some(user) = users::get_current_username() else {
        info!("Failed to get current user name. Session won't be unlocked.");
        return Message::Ignore;
    };

    let user = user.to_string_lossy();

    info!("Current user is \"{user}\".");

    client.conversation_mut().set_credentials(user, password);

    match client.authenticate() {
        Ok(_) => {
            info!("Password correct.");
            Message::Unlock
        }
        Err(_) => {
            info!("Password incorrect.");
            Message::WrongPassword
        }
    }
}
