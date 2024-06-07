use log::info;

use crate::app::Message;

#[must_use]
pub fn check_password(password: String) -> Message {
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

    if client.authenticate().is_ok() {
        info!("Password correct.");
        Message::Unlock
    } else {
        info!("Password incorrect.");
        Message::WrongPassword
    }
}
