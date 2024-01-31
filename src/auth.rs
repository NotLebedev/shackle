use log::info;
use users::get_current_username;

use crate::Message;

pub async fn check_password(password: String) -> Message {
    let Ok(mut client) = pam::Client::with_password("shackle") else {
        info!("Failed to initialize PAM client. Session won't be unlocked.");
        return Message::Ignore;
    };

    let Some(user) = get_current_username() else {
        info!("Failed to get current user name. Session won't be unlocked.");
        return Message::Ignore;
    };

    info!("Current user is \"{}\".", user.to_string_lossy());
    client
        .conversation_mut()
        .set_credentials(user.to_string_lossy(), password);
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

pub fn start_password_check(password: &String) -> iced::Command<Message> {
    return iced::Command::perform(check_password(password.clone()), |m| m);
}
