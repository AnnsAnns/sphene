use poise::serenity_prelude::{
    Context, CreateActionRow, CreateAllowedMentions, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, EditInteractionResponse, FullEvent, Interaction, InteractionType, Message, MessageBuilder, UserId
};
use rust_i18n::t;
use thorium::{bluesky, db::DBConn, instagram, tiktok, twitter};
use tokio::sync::Mutex;

use crate::utils::REGEX_URL;

pub async fn interaction_create(ctx: &Context, interaction: Interaction, dbconn: &Mutex<DBConn>) {
    // Check whether button has been pressed
    if interaction.kind() != InteractionType::Component {
        return;
    }

    let component = interaction.as_message_component().unwrap().clone();
    let command = &component.data.custom_id;
    let msg = &component.message;

    if !msg.author.bot {
        return;
    }

    // Get guild ID
    let id = msg.author.id.get();
    let get_lang = match dbconn.lock().await.get_server(id, false).language {
        Some(lang) => lang,
        None => "en".to_string(),
    };
    let lang = get_lang.as_str();

    let user = &component.user.id.to_string();
    // Check whether user is correct
    if !msg.content.contains(user)
        || command == "version"
        || command == "download"
        || command == "menu"
        || command == "disable"
        || command == "set_language"
        || command == "contribute_language"
    {
        let content = if command == "version" {
            t!(
                "source_code",
                locale = lang,
                URL = "https://github.com/AnnsAnns/sphene"
            )
            .to_string()
        } else if command == "menu" {
            t!("menu_meme").to_string()
        } else if command == "disable" {
            t!("disable").to_string()
        } else if command == "set_language" || command == "contribute_language" {
            t!(
                "contribute_language",
                locale = lang,
                URL = "https://github.com/AnnsAnns/sphene/locales"
            )
            .to_string()
        } else if command == "download" {
            let extracted_url = REGEX_URL
                .find_iter(&msg.content)
                .next()
                .unwrap()
                .as_str()
                .to_string();

            let url = if twitter::UrlType::from_string(&extracted_url)
                != twitter::UrlType::Unknown
            {
                twitter::get_media_from_url(
                    twitter::convert_url_lazy(extracted_url, twitter::UrlType::Vxtwitter).await,
                )
                .await
            } else if tiktok::UrlType::from_string(&extracted_url) != tiktok::UrlType::Unknown {
                tiktok::get_media_from_url(
                    tiktok::convert_url_lazy(extracted_url, tiktok::UrlType::TIKTXK).await,
                )
                .await
            } else {
                bluesky::get_media_from_url(
                    bluesky::convert_url_lazy(extracted_url, bluesky::UrlType::FixBluesky)
                        .await,
                )
                .await
            };

            if url != "0" {
                t!("download_url", locale = lang, URL = url).to_string()
            } else {
                t!("no_download", locale = lang).to_string()
            }
        } else {
            t!("not_author", locale = lang).to_string()
        };
        
        let response_msg = CreateInteractionResponseMessage::new().content(content).ephemeral(true);
        let response = CreateInteractionResponse::Defer(response_msg);

        component
            .create_response(&ctx.http, response)
            .await
            .unwrap();

        return;
    } else {
        // Make the Discord API happy :)
        component
            .create_response(&ctx.http, CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()))
            .await
            .unwrap();
    }

    if command == "remove" {
        let interaction_response = EditInteractionResponse::new()
            .content(t!("deleted_message", locale = lang))
            .allowed_mentions(CreateAllowedMentions::new().empty_roles().empty_users());

        if let Err(why) = component
            .edit_response(&ctx.http, interaction_response)
            .await
        {
            println!("Error editing message: {:?}", why);
        }

        // Sleep for 5 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Delete the response message
        if let Err(why) = component
            .delete_response(&ctx.http)
            .await
        {
            println!("Error deleting message: {:?}", why);
        }
    } else {
        let extracted_url = REGEX_URL
            .find_iter(&msg.content)
            .next()
            .unwrap()
            .as_str()
            .to_string();
        let mut new_msg: String = String::new();

        let mut twitter_urltype = twitter::UrlType::from_string(&command);
        let bluesky_urltype = bluesky::UrlType::from_string(&command);
        let instagram_urltype = instagram::UrlType::from_string(&command);
        let tiktok_urltype = tiktok::UrlType::from_string(&command);

        if twitter_urltype != twitter::UrlType::Unknown {
            new_msg = twitter::convert_url_lazy(extracted_url, twitter_urltype).await;
        } else if bluesky_urltype != bluesky::UrlType::Unknown {
            new_msg = bluesky::convert_url_lazy(extracted_url, bluesky_urltype).await;
        } else if instagram_urltype != instagram::UrlType::Unknown {
            new_msg = instagram::convert_url_lazy(extracted_url, instagram_urltype).await;
        } else if tiktok_urltype != tiktok::UrlType::Unknown {
            new_msg = tiktok::convert_url_lazy(extracted_url, tiktok_urltype).await;
        } else if command == "direct_vx" || command == "direct_fx" {
            twitter_urltype = match command.as_str() {
                "direct_vx" => twitter::UrlType::Vxtwitter,
                "direct_fx" => twitter::UrlType::Fxtwitter,
                _ => twitter::UrlType::Unknown,
            };

            new_msg =
                twitter::convert_url_lazy(extracted_url.to_string(), twitter_urltype).await;
            new_msg = format!(
                "<{}> ({})",
                new_msg,
                twitter::get_media_from_url(new_msg.clone()).await
            );
        } else if command == "direct_fxbsky" {
            new_msg = bluesky::convert_url_lazy(
                extracted_url.to_string(),
                bluesky::UrlType::FixBluesky,
            )
            .await;
            new_msg = format!(
                "<{}> ({})",
                new_msg,
                bluesky::get_media_from_url(new_msg.clone()).await
            );
        } else if command == "direct_tiktxk" {
            new_msg =
                tiktok::convert_url_lazy(extracted_url.to_string(), tiktok::UrlType::TIKTXK)
                    .await;
            new_msg = format!(
                "<{}> ({})",
                new_msg,
                tiktok::get_media_from_url(new_msg.clone()).await
            );
        }

        new_msg = format!("<@{}>: {}", user, new_msg);

        let new_response = EditInteractionResponse::new()
            .content(new_msg)
            .allowed_mentions(CreateAllowedMentions::new().empty_roles().empty_users());

        if let Err(why) = component
            .edit_response(&ctx.http,  new_response)
            .await
        {
            println!("Error editing message: {:?}", why);
        }
    }
}