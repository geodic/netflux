use crate::fetch::{MediaType, fetch};
use anyhow::Result;
use futures::future::{self, Map};
use semver::Version;
use std::net::Ipv4Addr;
use stremio_addon_sdk::builder::Builder;
use stremio_addon_sdk::scaffold::Scaffold;
use stremio_addon_sdk::server::{ServerOptions, serve_http};
use stremio_core::state_types::EnvFuture;
use stremio_core::types::addons::*;
use stremio_core::types::*;

fn handle_stream(resource: &ResourceRef) -> EnvFuture<ResourceResponse> {
    let mut final_streams = vec![];
    let media_type = match resource.type_name.as_str() {
        "movie" => MediaType::Movie,
        "series" => MediaType::Series,
        _ => panic!("Unsupported resource type: {}", resource.type_name),
    };

    if let Ok(streams) = fetch(media_type, &resource.id) {
        for stream in streams {
            final_streams.push(Stream {
                title: Some(stream.quality),
                source: StreamSource::Url { url: stream.url },
                thumbnail: None,
                subtitles: vec![],
                behavior_hints: Default::default(),
            });
        }
    }

    return Box::new(future::ok(ResourceResponse::Streams {
        streams: final_streams,
    }));
}

pub fn serve(port: u16) -> Result<()> {
    let manifest = Manifest {
        id: "org.netflux".into(),
        name: "Netflux".into(),
        version: Version::new(0, 1, 0),
        resources: vec![ManifestResource::Short("stream".into())],
        types: vec!["movie".into(), "series".into()],
        id_prefixes: Some(vec!["tt".into()]),
        description: Some("Netflix, but free".into()),
        logo: Some("https://raw.githubusercontent.com/geodic/netflux/main/assets/logo.png".into()),
        ..Scaffold::default_manifest()
    };
    let builder = Builder::new(manifest)
        .define_stream_handler(handle_stream)
        .build();
    let options: ServerOptions = ServerOptions {
        port: port,
        cache_max_age: 0,
        ip: Ipv4Addr::new(0, 0, 0, 0).into(),
    };
    serve_http(builder, options);
    Ok(())
}
