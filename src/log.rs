use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn setup_logger() -> color_eyre::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = fmt::layer().pretty().with_writer(std::io::stderr);

    // let file_appender = rolling::daily("logs", "app.log");
    // let (non_blocking_appender, _guard) = non_blocking(file_appender);
    // let file_layer = fmt::layer()
    // .with_ansi(false)
    // .with_writer(non_blocking_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(ErrorLayer::default())
        .with(formatting_layer)
        // .with(file_layer)
        .init();

    color_eyre::install()?;
    Ok(())
}
