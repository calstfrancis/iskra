mod commands;
mod config;
mod error;
mod library;
mod model;
mod rcl;
mod state;
mod storage;
mod ui;

use std::env;

use glib::ExitCode;
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use libadwaita as adw;

use config::Config;
use ui::app_window::AppWindow;

fn main() -> ExitCode {
    let log_dir = glib::user_data_dir().join("iskra");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::never(&log_dir, "iskra.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Iskra starting — log: {}", log_dir.join("iskra.log").display());

    if env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "Iskra — sermon planning for preachers who work from structured notes\n\
             \n\
             Usage: iskra [OPTIONS]\n\
             \n\
             Options:\n\
               -h, --help     Print this help message\n\
               --version      Print version\n\
             \n\
             Configuration: ~/.config/iskra/config.toml\n\
             Log file:       ~/.local/share/iskra/iskra.log\n\
             Sermons:        set via config.toml work_dir key (default ~/Documents/Iskra)"
        );
        return ExitCode::SUCCESS;
    }
    if env::args().any(|a| a == "--version") {
        println!("iskra {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let app = adw::Application::new(
        Some("io.github.calstfrancis.Iskra"),
        ApplicationFlags::empty(),
    );

    app.connect_activate(move |app| {
        let config = Config::load().unwrap_or_default();
        let window = AppWindow::new(app, config);
        window.present();
    });

    app.run_with_args(&env::args().collect::<Vec<_>>())
}
