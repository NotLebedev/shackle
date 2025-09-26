use std::{cell::Cell, env, fs, path::PathBuf};

use gtk::gio::{self, prelude::ApplicationExtManual};
use log::error;
use nix::fcntl::{Flock, FlockArg};

pub struct Lock {
    _lock: Flock<fs::File>,
}

/// Ensure that this is the only running instance
/// by acquiring an exclusive lock
pub fn lock_sole_instance() -> Option<Lock> {
    let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") else {
        error!("XDG_RUNTIME_DIR not set. Is your session running?");
        return None;
    };

    let mut tmp_dir = PathBuf::from(runtime_dir);
    tmp_dir.push("shackle");

    let mut lock_file = tmp_dir.clone();
    lock_file.push("shackle.lock");

    let _ = fs::create_dir_all(&tmp_dir);

    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_file)
        .ok()?;

    let lock = Flock::lock(lock_file, FlockArg::LockExclusiveNonblock).ok()?;

    lock.try_lock().ok()?;

    Some(Lock { _lock: lock })
}

pub struct AppHold {
    hold: Cell<Option<gio::ApplicationHoldGuard>>,
}

impl AppHold {
    pub fn new(app: &gtk::Application) -> Self {
        Self {
            hold: Cell::new(Some(app.hold())),
        }
    }

    /// Release associated [`gio::ApplicationHoldGuard`]
    ///
    /// This operation is idempotent
    pub fn release(&self) {
        if let Some(hold) = self.hold.replace(None) {
            drop(hold);
        }
    }
}
