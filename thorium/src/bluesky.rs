use tokio::task::spawn_blocking;

use crate::USER_AGENT;

pub const BLUESKY_URL: &str = "https://bsky.app/";
pub const PSKY_URL: &str = "https://psky.app/";
pub const FIXBLUESKY_URL: &str = "https://bsyy.app/";

#[derive(Debug, PartialEq)]
pub enum UrlType {
    Bluesky,
    Psky,
    FixBluesky,
    Unknown,
}

impl UrlType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UrlType::Bluesky => BLUESKY_URL,
            UrlType::Psky => PSKY_URL,
            UrlType::FixBluesky => FIXBLUESKY_URL,
            UrlType::Unknown => "",
        }
    }

    pub fn from_string(url: &str) -> UrlType {
        if url.contains(BLUESKY_URL) {
            return UrlType::Bluesky;
        } else if url.contains(PSKY_URL) {
            return UrlType::Psky;
        } else if url.contains(FIXBLUESKY_URL) {
            return UrlType::FixBluesky;
        }
        UrlType::Unknown
    }
}

pub fn is_bluesky_url(url: &str) -> bool {
    UrlType::from_string(url) == UrlType::Bluesky
}

pub async fn convert_url(url: String, from: UrlType, to: UrlType) -> String {
    url.replace(from.as_str(), to.as_str())
}

pub async fn convert_url_lazy(url: String, to: UrlType) -> String {
    let from = UrlType::from_string(&url);
    url.replace(from.as_str(), to.as_str())
}

pub async fn get_url_type(url: String) -> UrlType {
    UrlType::from_string(&url)
}

pub async fn get_media_from_url(mut url: String) -> String {
    url = convert_url_lazy(url, UrlType::FixBluesky).await;

    // We don't want to follow the redirect so we can get the metadata
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap();

    let request = client
        .get(&url)
        .header("user-agent", USER_AGENT)
        .send()
        .await
        .unwrap();

    let content = request.text().await.unwrap();

    // Check if content has a meta property and return it in a blocking thread
    url = spawn_blocking(move || {
        let selector_img = scraper::Selector::parse("meta[property='og:image']").unwrap();
        let selector_video = scraper::Selector::parse("meta[property='og:video']").unwrap();
        let html = scraper::Html::parse_document(content.as_str());
        let vid = html.select(&selector_video).next();
        let img = html.select(&selector_img).next();
        if vid.is_none() && img.is_none() {
            return "0".to_string();
        }

        let url = if let Some(vid) = vid {
            vid
        } else {
            img.unwrap()
        };

        url.value().attr("content").unwrap().to_string()
    })
    .await
    .unwrap();
    url
}
