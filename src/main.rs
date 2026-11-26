// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod config;
mod error;
mod features;
mod i18n;
mod shared;

pub use error::*;

use cosmic::{app::Settings, iced::Limits};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::{AppModel, Flags};

fn main() -> Result<()> {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    // Initialize tracing for logging and debugging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("tasks=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let flags = Flags::load()?;
    let settings = Settings::default()
        .theme(flags.config.app_theme.theme())
        .size_limits(Limits::NONE.min_width(350.0).min_height(180.0))
        .debug(false);

    cosmic::app::run::<AppModel>(settings, flags).map_err(Error::Iced)
}
