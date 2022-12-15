use cf::{
    display::tui::{app::App, view::MainBrowser},
    log::setup_logger,
};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger()?;
    let mut app = App::new()?;
    app.enter_new_view(Box::new(MainBrowser::new()?));
    app.run()?;
    Ok(())
}
