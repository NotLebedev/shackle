use clap::Parser;
use fork::daemon;
use fork::Fork;
use gtk::gdk;
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk4_session_lock::Instance as SessionLockInstance;
use log::{error, info};

use crate::fprint::check_fingerprint;
use crate::ui::controls;

mod fprint;
mod pam;
mod ui;

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

    let controls = controls(lock);

    window.set_child(Some(&controls));

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
            if check_fingerprint(false).await {
                lock.unlock();
            }
        }
    ));

    glib::spawn_future_local(clone!(
        #[weak]
        lock,
        async move {
            glib::unix_signal_future(nix::sys::signal::Signal::SIGUSR1 as i32).await;
            info!("Recieved SIGUSR1.");
            lock.unlock();
        }
    ));

    // When this function exits session is not guaranteed to be locked
    lock.lock();
}

fn main() {
    let args = Args::parse();

    if args.daemonize {
        if let Ok(Fork::Child) = daemon(true, true) {
            start(args);
        }
    } else {
        start(args);
    }
}

fn start(_args: Args) {
    env_logger::init();
    let _ = gtk::init();

    if !gtk4_session_lock::is_supported() {
        error!("Session lock not supported");
        std::process::exit(1);
    }

    let app = gtk::Application::new(Some("org.notlebedev.shackle"), Default::default());

    app.connect_activate(activate);
    app.run_with_args(&Vec::<String>::new());
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Fork off locker process
    #[arg(short, long)]
    daemonize: bool,
    /// Start fingerprint verification only after device wakes up
    ///
    /// Useful if fingerprint verification does not work (or is delayed)
    /// after devices goes to sleep
    #[arg(short, long)]
    await_wakeup: bool,
}
