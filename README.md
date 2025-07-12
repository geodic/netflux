## I am archiving this repository in favor of better addons such as [Nuvio Streams](https://nuviostreams.hayd.uk/)

# Netflux

A Stremio addon that provides streaming content. Built with Rust for high performance and reliability.

This addon should be able to stream most English-language shows and movies. However, it is still in development.

If you find that a show or movie is not available, please double-check its availability on the Vidsrc website:

- For movies, go to: `https://vidsrc.xyz/embed/movie/<imdb_id>`
- For series, go to: `https://vidsrc.xyz/embed/tv/<imdb_id>/<season>-<episode>`
  - Example: For IMDb ID `tt1234567`, season 1, episode 2: `https://vidsrc.xyz/embed/tv/tt1234567/1-2`

If it is available there but not in this addon, open an issue and include the IMDb ID (and season/episode for series) of the missing content.

## Features

- Movie streaming support
- Series streaming support
- Compatible with Stremio's addon system

## Prerequisites

- Docker (recommended)
- Or Rust 1.75+ for local development

## Quick Start

### Using Docker

```bash
docker pull ghcr.io/geodic/netflux:latest
docker run -p 80:80 ghcr.io/geodic/netflux:latest
```

### Local Development

1. Install Rust (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone the repository:
```bash
git clone https://github.com/geodic/netflux.git
cd netflux
```

3. Build and run:
```bash
cargo build --release
./target/release/netflux
```

## Environment Variables

- `PORT`: Server port (default: 80)
- `CHROME_PATH`: Path to Chrome/Chromium executable (automatically set in Docker)
- `RUST_LOG`: Rust logging level (default: error)

## Adding to Stremio

1. Start the addon server (either via Docker or locally)
2. In Stremio, go to the addons page
3. Click "Enter addon URL"
4. Enter: `http://localhost:80/manifest.json` (for local development) or `https://your-domain/manifest.json` (for remote deployment; see note below)

**Note:** If you are not using `localhost`, Stremio requires the addon to be served over HTTPS. This project does not support HTTPS natively, so you must use a reverse proxy (such as Nginx or Caddy) to provide HTTPS support.

## Development

The project is structured as follows:
- `src/main.rs`: Application entry point
- `src/stremio.rs`: Stremio addon implementation
- `src/fetch.rs`: Content fetching logic

### Building

```bash
cargo build          # Debug build
cargo build --release  # Release build
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT

## Acknowledgments

- Built with [Stremio Addon SDK](https://github.com/Stremio/stremio-addon-sdk)
- Uses [headless_chrome](https://crates.io/crates/headless_chrome) for content fetching

## Disclaimer

This project is provided as-is, without any warranty. The author assumes no responsibility or liability for any use, misuse, or distribution of this software. By using this software, you agree that the author is not liable for any legal, civil, or criminal consequences that may arise from its use.
