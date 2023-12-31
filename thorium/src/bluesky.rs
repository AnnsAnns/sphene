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
    crate::get_media(url).await
}
