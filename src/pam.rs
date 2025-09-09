use std::ffi::{OsStr, OsString};

use log::{info, warn};
use nonstick::{
    AuthnFlags, ConversationAdapter, Result as PamResult, Transaction, TransactionBuilder,
};

struct UsernamePassConvo {
    username: String,
    password: String,
}

impl ConversationAdapter for UsernamePassConvo {
    fn prompt(&self, _request: impl AsRef<OsStr>) -> PamResult<OsString> {
        Ok(OsString::from(&self.username))
    }

    fn masked_prompt(&self, _request: impl AsRef<OsStr>) -> PamResult<OsString> {
        Ok(OsString::from(&self.password))
    }

    fn error_msg(&self, _message: impl AsRef<OsStr>) {}

    fn info_msg(&self, _message: impl AsRef<OsStr>) {}
}

/// This function is blocking. Run it using [`gtk::gio::spawn_blocking`]
pub fn check_password(password: String) -> bool {
    info!("Starting pam authentification.");
    let Some(username) =
        users::get_current_username().map(|os_string| os_string.to_string_lossy().into_owned())
    else {
        warn!("Failed to get current user name. Session won't be unlocked.");
        return false;
    };

    info!("Current user is \"{username}\".");

    let credentials = UsernamePassConvo {
        username: username.clone(),
        password,
    };

    let Ok(mut txn) = TransactionBuilder::new_with_service("shackle")
        .username(username)
        .build(credentials.into_conversation())
    else {
        warn!("Failed to initialize PAM client. Session won't be unlocked.");
        return false;
    };

    match txn.authenticate(AuthnFlags::empty()) {
        Ok(_) => {
            info!("Password correct.");
            true
        }
        Err(_) => {
            info!("Password incorrect.");
            false
        }
    }
}
