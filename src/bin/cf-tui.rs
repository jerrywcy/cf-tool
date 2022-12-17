use cf::{
    display::tui::{app::App, msg::ViewConstructor},
    log::setup_logger,
    settings::load_settings,
};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    load_settings()?;
    let mut app = App::new()?;
    app.enter_new_view(ViewConstructor::MainBrowser);
    if let Err(err) = app.run() {
        drop(app);
        eprintln!("{err:#}");
    }
    Ok(())
}
