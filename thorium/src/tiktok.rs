pub const TIKTOK_URL: &str = "https://www.tiktok.com/";
pub const TIKTOK_URL_TK: &str = "https://vt.tiktok.com/";
pub const TIKTOK_NWWW_URL: &str = "https://tiktok.com/";
pub const TNKTOK_URL: &str = "https://tnktok.com/";
pub const TIKTXK_URL: &str = "https://tiktxk.com/";

#[derive(Debug, PartialEq)]
pub enum UrlType {
    TikTok,
    TNKTOK,
    TIKTXK,
    Unknown
}

impl UrlType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UrlType::TikTok => TIKTOK_URL,
            UrlType::TIKTXK => TIKTXK_URL,
            UrlType::TNKTOK => TNKTOK_URL,
            UrlType::Unknown => "",
        }
    }

    pub fn from_string(url: &str) -> UrlType {
        if url.contains(TIKTOK_URL) || url.contains(TIKTOK_NWWW_URL) || url.contains(TIKTOK_URL_TK) {
            return UrlType::TikTok;
        } else if url.contains(TNKTOK_URL) {
            return UrlType::TNKTOK;
        } else if url.contains(TIKTXK_URL) {
            return UrlType::TIKTXK;
        }
        UrlType::Unknown
    }
}

pub fn is_tiktok_url(url: &str) -> bool {
    UrlType::from_string(url) == UrlType::TikTok
}

pub async fn convert_url(url: String, from: UrlType, to: UrlType) -> String {
    return clear_url(url).await.replace(from.as_str(), to.as_str());
}

pub async fn convert_url_lazy(url: String, to: UrlType) -> String {
    let cleared_url = clear_url(url.clone()).await;
    let from = UrlType::from_string(&cleared_url);
    url.replace(from.as_str(), to.as_str())
}

/**
 * Remove vt. from the URL to get the original URL
 */
pub async fn clear_url(url: String) -> String {
    if url.contains("vt.") {
        // Remove vt. from the URL
        return url.replace("vt.", "");
    } else if !url.contains("www.") {
        // Append www. to the URL
        return url.replace("tiktok.com", "www.tiktok.com");
    }
    url
}

pub async fn get_url_type(url: String) -> UrlType {
    UrlType::from_string(&url)
}

pub async fn get_media_from_url(mut url: String) -> String {
    url = convert_url_lazy(url, UrlType::TIKTXK).await;
    crate::get_media(url).await
}
