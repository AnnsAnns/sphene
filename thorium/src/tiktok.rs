pub const TIKTOK_URL: &str = "https://www.tiktok.com/";
pub const TIKTOK_NWWW_URL: &str = "https://tiktok.com/";
pub const VXTIKTOK_URL: &str = "https://vxtiktok.com/";

#[derive(Debug, PartialEq)]
pub enum UrlType {
    TikTok,
    VXTikTok,
    Unknown
}

impl UrlType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UrlType::TikTok => TIKTOK_URL,
            UrlType::VXTikTok => VXTIKTOK_URL,
            UrlType::Unknown => "",
        }
    }

    pub fn from_string(url: &str) -> UrlType {
        if url.contains(TIKTOK_URL) || url.contains(TIKTOK_NWWW_URL) {
            return UrlType::TikTok;
        } else if url.contains(VXTIKTOK_URL) {
            return UrlType::VXTikTok;
        }
        UrlType::Unknown
    }
}

pub fn is_tiktok_url(url: &str) -> bool {
    UrlType::from_string(url) == UrlType::TikTok
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
    url = convert_url_lazy(url, UrlType::VXTikTok).await;
    crate::get_media(url).await
}
