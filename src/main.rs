mod fetch;
mod stremio;
use anyhow::Result;
use stremio::serve;

fn main() -> Result<()> {
    env_logger::init();
    serve(
        std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(80),
    )?;
    Ok(())
}
