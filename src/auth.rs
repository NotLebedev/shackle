use log::info;
use users::get_current_username;

pub fn check_password(password: &str) -> bool {
    let Ok(mut client) = pam::Client::with_password("shackle") else {
        info!("Failed to initialize PAM client. Session won't be unlocked.");
        return false;
    };

    let Some(user) = get_current_username() else {
        info!("Failed to get current user name. Session won't be unlocked.");
        return false;
    };

    info!("Current user is \"{}\".", user.to_string_lossy());
    client
        .conversation_mut()
        .set_credentials(user.to_string_lossy(), password);
    client.authenticate().is_ok()
}
