use color_eyre::Report;

use tracing_subscriber::EnvFilter;
use ui::MCubedAppUI;

use eframe::{egui::Vec2, IconData};

mod ui;

fn main() -> Result<(), Report> {
    setup_logging()?;

    let app_icon = load_icon_data(include_bytes!("../res/app_icon.png"));
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(970., 300.)),
        min_window_size: Some(Vec2::new(600., 300.)),
        icon_data: Some(app_icon),
        ..Default::default()
    };

    eframe::run_native(
        "mCubed",
        native_options,
        Box::new(|cc| Box::new(MCubedAppUI::new(cc))),
    );
}

fn setup_logging() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let env_filter = EnvFilter::try_from_default_env()?.add_directive("back=info".parse()?);

    tracing_subscriber::fmt::fmt()
        .with_env_filter(env_filter)
        .init();

    Ok(())
}

pub fn load_icon_data(image_data: &[u8]) -> IconData {
    let image = image::load_from_memory(image_data).unwrap();
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_raw().clone();

    IconData {
        rgba: pixels,
        width: image.width(),
        height: image.height(),
    }
}
