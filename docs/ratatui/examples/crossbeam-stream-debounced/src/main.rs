use color_eyre::Result;
use crossbeam_stream_debounced::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut app = App::new().await?;
    app.run().await?;
    Ok(())
}
