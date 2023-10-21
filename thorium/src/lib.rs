use tokio::task::spawn_blocking;

pub mod twitter;
pub mod bluesky;
pub mod instagram;
pub mod tiktok;
pub mod db;

const USER_AGENT: &str = "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)";

pub async fn get_media(mut url: String) -> String {
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