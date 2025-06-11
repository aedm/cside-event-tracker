mod storage;
mod event;
mod server;

use anyhow::Result;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    set_up_tracing()?;
    server::serve().await?;
    Ok(())
}

fn set_up_tracing() -> Result<()> {
    #[cfg(windows)]
    let with_color = nu_ansi_term::enable_ansi_support().is_ok();
    #[cfg(not(windows))]
    let with_color = true;

    // let crate_filter =
    //     tracing_subscriber::filter::filter_fn(|metadata| metadata.target().starts_with("bitang"));
    let fmt_layer = fmt::layer().with_ansi(with_color).with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(if cfg!(debug_assertions) { "debug" } else { "info" }))?;
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        // .with(crate_filter)
        .init();

    Ok(())
}
