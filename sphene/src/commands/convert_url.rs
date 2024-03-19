use poise::serenity_prelude::{CreateSelectMenuOption, Message};
use thorium::db::DBConn;
use tokio::sync::Mutex;



use thorium::{bluesky, instagram, tiktok, twitter};

use crate::{
    options::{
        get_blueksy_options, get_instagram_options, get_tik_tok_options, get_twitter_options,
    },
};

pub struct ConvertedUrl {
    pub url: String,
    pub options: Vec<CreateSelectMenuOption>,
}

pub async fn convert_url(msg: Message, dbconn: &Mutex<DBConn>, id: u64, lang: &str) -> Option<ConvertedUrl> {
    let url: String;
    let content = msg.content.clone();
    let options: Vec<CreateSelectMenuOption>;

    if twitter::is_twitter_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).twitter
    {
        url = twitter::remove_tracking(
            twitter::convert_url_lazy(content, twitter::UrlType::Vxtwitter).await,
        );

        options = get_twitter_options(lang);
    } else if bluesky::is_bluesky_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).bluesky
    {
        url = bluesky::convert_url_lazy(content, bluesky::UrlType::FixBluesky).await;
        options = get_blueksy_options(lang);
    } else if tiktok::is_tiktok_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).tiktok
    {
        url = tiktok::convert_url_lazy(tiktok::clear_url(content).await, tiktok::UrlType::TIKTXK)
            .await;
        options = get_tik_tok_options(lang);
    } else if instagram::is_instagram_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).instagram
    {
        url = instagram::convert_url_lazy(content, instagram::UrlType::DDInstagram).await;
        options = get_instagram_options(lang);
    } else {
        return None;
    }
    
    Some(ConvertedUrl { url, options })
}

pub async fn convert_twitter_to(url: String, kind: &str) -> String {
    let twitter_urltype = match kind {
        "direct_vx" => twitter::UrlType::Vxtwitter,
        "direct_fx" => twitter::UrlType::Fxtwitter,
        _ => twitter::UrlType::Unknown,
    };

    let new_msg =
        twitter::convert_url_lazy(url.to_string(), twitter_urltype).await;
    
    format!(
        "<{}> ({})",
        new_msg,
        twitter::get_media_from_url(new_msg.clone()).await
    )
}