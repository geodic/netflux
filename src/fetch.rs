use anyhow::Result;
use headless_chrome::{
    Browser,
    browser::{
        LaunchOptions,
        tab::{RequestInterceptor, RequestPausedDecision},
        transport::{SessionId, Transport},
    },
    protocol::cdp::Fetch::events::RequestPausedEvent,
};
use hls_m3u8::MasterPlaylist;
use itertools::Itertools;
use log::{debug, error, info, warn};
use regress::Regex;
use scraper::{Html, Selector};
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
    vec,
};
use url::Url;

#[derive(Default)]
struct Interceptor {
    video_url: (Mutex<String>, Condvar),
    master: Mutex<bool>,
}

struct Playlist {
    url: String,
    master: bool,
}

impl RequestInterceptor for Interceptor {
    fn intercept(
        &self,
        _transport: Arc<Transport>,
        _session_id: SessionId,
        event: RequestPausedEvent,
    ) -> RequestPausedDecision {
        let url = event.params.request.url;
        if url.contains("m3u8") {
            let mut master = self.master.lock().unwrap();
            let response = reqwest::blocking::get(url.clone())
                .expect("Failed to fetch m3u8 file")
                .text()
                .expect("Failed to read m3u8 response");
            *master = MasterPlaylist::try_from(response.as_str()).is_ok();
            let (lock, cvar) = &self.video_url;
            let mut video_url = lock.lock().unwrap();
            *video_url = url;
            cvar.notify_all();
        }
        RequestPausedDecision::Continue(None)
    }
}

#[derive(Debug)]
pub struct Stream {
    pub url: String,
    pub quality: String,
}

pub enum MediaType {
    Series,
    Movie,
}

fn extract_stream(base_url: &Url, stream: hls_m3u8::tags::VariantStream) -> Stream {
    match stream {
        hls_m3u8::tags::VariantStream::ExtXStreamInf {
            stream_data, uri, ..
        } => Stream {
            url: base_url.join(&uri).unwrap().to_string(),
            quality: format!("{}p", stream_data.resolution().unwrap().height()),
        },
        hls_m3u8::tags::VariantStream::ExtXIFrame { .. } => {
            panic!("Stream not supported")
        }
    }
}

pub fn fetch(media_type: MediaType, imdb_id: &str) -> Result<Vec<Stream>> {
    let url = Url::parse("https://vidsrc.xyz/embed/")?;
    let url = match media_type {
        MediaType::Series => {
            let mut url = url.join("tv/")?;
            let (imdb_id, season, episode) = imdb_id.split(":").collect_tuple().unwrap();
            url = url.join(format!("{}/", imdb_id).as_str())?;
            url = url.join(format!("{}-{}", season, episode).as_str())?;
            url
        }
        MediaType::Movie => url.join("movie/")?.join(imdb_id)?,
    };

    info!("Trying to fetch streams for {} using request", imdb_id);
    let playlist = fetch_request(&url).or_else(|e| {
        error!("Failed to fetch streams using request: {}", e);
        error!("Falling back to browser fetch");
        info!("Trying to fetch streams for {} using browser", imdb_id);
        fetch_browser(url.as_str())
    })?;

    let Playlist {
        url: video_url,
        master,
    } = playlist;
    let video_url = Url::parse(&video_url)?;

    let streams: Vec<Stream>;
    if master {
        let m3u8 = reqwest::blocking::get(video_url.clone())?.text()?;
        let variant_streams = MasterPlaylist::try_from(m3u8.as_str())?.variant_streams;
        streams = variant_streams
            .into_iter()
            .map(|stream| extract_stream(&video_url, stream))
            .collect::<Vec<Stream>>();
    } else {
        streams = vec![Stream {
            url: video_url.to_string(),
            quality: "Unknown".to_string(),
        }];
    }

    Ok(streams)
}

pub fn fetch_request(url: &Url) -> Result<Playlist> {
    let client = reqwest::blocking::Client::new();

    let response = client.get(url.as_str()).send()?.text()?;
    let dom = Html::parse_document(&response);
    let selector = Selector::parse("iframe#player_iframe").unwrap();
    let url = url.join(dom.select(&selector).next().unwrap().attr("src").unwrap())?;

    let response = client.get(url.as_str()).send()?.text()?;
    let selector = Regex::new(r#"\/prorcp\/[^'"]*"#).unwrap();
    let url = url.join(
        &response[selector
            .find(&response)
            .ok_or_else(|| anyhow::anyhow!("Failed to find prorcp url"))?
            .group(0)
            .unwrap()],
    )?;
    info!("Found prorcp url: {}", url);

    let response = client.get(url.as_str()).send()?.text()?;
    let selector = Regex::new(r#"(?<!\/\/v.*)(?<=file:\s*['"])[^'"]+"#).unwrap();
    let url = &response[selector
        .find(&response)
        .ok_or_else(|| anyhow::anyhow!("Failed to find video url"))?
        .group(0)
        .unwrap()];
    info!("Found media url: {}", url);

    let response = client.get(url).send()?.text()?;
    let master = MasterPlaylist::try_from(response.as_str()).is_ok();

    if master {
        info!("Found master playlist");
    } else {
        info!("Found single stream");
    }

    Ok(Playlist {
        url: url.to_string(),
        master,
    })
}

pub fn fetch_browser(url: &str) -> Result<Playlist> {
    let browser = Browser::new(LaunchOptions {
        sandbox: false,
        ..Default::default()
    })?;
    // let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    tab.disable_debugger()?;

    let request_interceptor = Arc::new(Interceptor::default());
    let interceptor_clone = Arc::clone(&request_interceptor);

    tab.navigate_to(url)?.wait_until_navigated()?;

    let url = &tab
        .wait_for_element("#player_iframe")?
        .get_attribute_value("src")?
        .unwrap();
    let url = &format!("http:{}", url);

    info!("Found hash url: {}", url);

    tab.navigate_to(url)?.wait_until_navigated()?;

    tab.enable_fetch(None, None)?;
    tab.enable_request_interception(interceptor_clone)?;

    tab.wait_for_element("#pl_but")?.click()?;

    let (lock, cvar) = &request_interceptor.video_url;
    let video_url = lock.lock().unwrap();
    let (video_url, time_out) = cvar
        .wait_timeout(video_url, Duration::from_secs(10))
        .unwrap();
    if time_out.timed_out() {
        error!("Timed out waiting for video URL");
        anyhow::bail!("Timed out waiting for video URL");
    }
    let video_url = Url::parse(&*video_url)?;
    let master = request_interceptor.master.lock().unwrap();

    info!("Found media url: {}", video_url.clone());

    Ok(Playlist {
        url: video_url.to_string(),
        master: *master,
    })
}
