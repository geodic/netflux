mod fetch;
mod stremio;
use anyhow::Result;
use clap::Parser;
use stremio::serve;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 80)]
    port: u16,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    serve(
        std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(cli.port),
    )?;
    Ok(())
}
