pub const INSTAGRAM_URL: &str = "https://www.instagram.com/";
pub const INSTAGRAM_NWWW_URL: &str = "https://www.instagram.com/";
pub const DDINSTAGRAM_URL: &str = "https://www.ddinstagram.com/";

#[derive(Debug, PartialEq)]
pub enum UrlType {
    Instagram,
    DDInstagram,
    Unknown
}

impl UrlType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UrlType::Instagram => INSTAGRAM_URL,
            UrlType::DDInstagram => DDINSTAGRAM_URL,
            UrlType::Unknown => "",
        }
    }

    pub fn from_string(url: &str) -> UrlType {
        if url.contains(INSTAGRAM_URL) || url.contains(INSTAGRAM_NWWW_URL) {
            return UrlType::Instagram;
        } else if url.contains(DDINSTAGRAM_URL) {
            return UrlType::DDInstagram;
        }
        UrlType::Unknown
    }
}

pub fn is_instagram_url(url: &str) -> bool {
    UrlType::from_string(url) == UrlType::Instagram
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
    url = convert_url_lazy(url, UrlType::DDInstagram).await;
    crate::get_media(url).await
}
