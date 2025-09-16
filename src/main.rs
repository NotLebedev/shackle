mod auth;
mod config;
mod sole_instance;
mod ui;

use fork::daemon;
use fork::Fork;
use gtk::gdk;
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk4_session_lock::Instance as SessionLockInstance;
use log::warn;
use log::{error, info};

use crate::auth::fprint::check_fingerprint;
use crate::auth::signal::wait_signal;
use crate::config::config;
use crate::sole_instance::lock_sole_instance;
use crate::ui::background;
use crate::ui::controls;
use crate::ui::load_css;
use crate::ui::set_gtk_settings;

fn on_session_locked(_: &SessionLockInstance) {
    info!("Session locked successfully");
}

fn on_session_lock_failed(app: &gtk::Application) {
    error!("The session could not be locked");
    app.quit();
}

fn on_session_unlocked(app: &gtk::Application) {
    info!("Session unlocked");
    app.quit();
}

fn on_monitor_present(lock: &SessionLockInstance, monitor: gdk::Monitor, app: &gtk::Application) {
    // TODO: this function creates ui on each monitor. We need to present controls only on one
    // and just beatuiful background on rest

    let window = gtk::ApplicationWindow::new(app);

    let bg_overlay = gtk::Overlay::new();
    bg_overlay.set_child(Some(&background()));
    bg_overlay.add_overlay(&controls(lock));

    window.set_child(Some(&bg_overlay));

    lock.assign_window_to_monitor(&window, &monitor);
    // No need for window.present
    // gtk_session_lock_instance_assign_window_to_monitor() does that
}

fn activate(app: &gtk::Application) {
    let lock = SessionLockInstance::new();
    lock.connect_locked(on_session_locked);
    lock.connect_failed(clone!(
        #[weak]
        app,
        move |_| on_session_lock_failed(&app)
    ));

    lock.connect_unlocked(clone!(
        #[weak]
        app,
        move |_| on_session_unlocked(&app)
    ));

    lock.connect_monitor(clone!(
        #[weak]
        app,
        move |lock, monitor| on_monitor_present(&lock, monitor.clone(), &app)
    ));

    glib::spawn_future_local(clone!(
        #[weak]
        lock,
        async move {
            if check_fingerprint(config().await_wakeup).await {
                lock.unlock();
            }
        }
    ));

    glib::spawn_future_local(clone!(
        #[weak]
        lock,
        async move {
            wait_signal().await;
            lock.unlock();
        }
    ));

    // When this function exits session is not guaranteed to be locked
    lock.lock();
}

fn main() {
    if config().daemonize {
        if let Ok(Fork::Child) = daemon(true, true) {
            start();
        }
    } else {
        start();
    }
}

fn start() {
    env_logger::init();

    let Some(_lock) = lock_sole_instance() else {
        warn!("Another instance of shackle is running. Terminating");
        return;
    };

    let _ = gtk::init();

    if !gtk4_session_lock::is_supported() {
        error!("Session lock not supported");
        std::process::exit(1);
    }

    let app = gtk::Application::new(Some("org.notlebedev.shackle"), Default::default());

    app.connect_startup(|_| {
        set_gtk_settings();
        load_css()
    });
    app.connect_activate(activate);
    app.run_with_args(&Vec::<String>::new());
}
