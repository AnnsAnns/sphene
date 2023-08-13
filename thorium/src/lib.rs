use tokio::task::spawn_blocking;

pub const TWITTER_URL: &str = "https://twitter.com/";
pub const X_URL: &str = "https://x.com/";
const FXTWITTER_URL: &str = "https://fxtwitter.com/";
const VXTWITTER_URL: &str = "https://vxtwitter.com/";
const MOSAIC_URL: &str = "https://mosaic.fxtwitter.com/";
const USER_AGENT: &str = "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)";

#[derive(Debug, PartialEq)]
pub enum UrlType {
    Twitter,
    X,
    Fxtwitter,
    Vxtwitter,
    Mosaic,
    Unknown,
}

impl UrlType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UrlType::Twitter => TWITTER_URL,
            UrlType::X => X_URL,
            UrlType::Fxtwitter => FXTWITTER_URL,
            UrlType::Vxtwitter => VXTWITTER_URL,
            UrlType::Mosaic => MOSAIC_URL,
            UrlType::Unknown => "",
        }
    }

    pub fn from_string(url: &str) -> UrlType {
        if url.contains(TWITTER_URL) {
            return UrlType::Twitter;
        } else if url.contains(X_URL) {
            return UrlType::X;
        } else if url.contains(FXTWITTER_URL) {
            return UrlType::Fxtwitter;
        } else if url.contains(VXTWITTER_URL) {
            return UrlType::Vxtwitter;
        } else if url.contains(MOSAIC_URL) {
            return UrlType::Mosaic;
        }
        UrlType::Unknown
    }
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

pub fn is_twitter_url(url: &str) -> bool {
    url.contains(TWITTER_URL) || url.contains(X_URL)
}

pub async fn get_media_from_url(mut url: String) -> String {
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
        let selector_img = scraper::Selector::parse("meta[property='twitter:image']").unwrap();
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

    // Check if it's a mosaic
    if url.contains(MOSAIC_URL) {
        url.push_str(".jpg")
    }
    url
}
