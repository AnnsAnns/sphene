use std::env;

extern crate dotenv;

use dotenv::dotenv;

use options::get_blueksy_options;
use options::get_instagram_options;
use options::get_tik_tok_options;
use options::get_twitter_options;
use regex::Regex;
use rust_i18n::available_locales;
use serenity::async_trait;
use serenity::builder::CreateSelectMenuOption;

use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::InteractionType;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Activity;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use thorium::db::DBConn;
use thorium::*;

use rust_i18n::t;

rust_i18n::i18n!("../locales", fallback = "en");

mod options;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Handler {
    regex_pattern: Regex,
    dbconn: Mutex<db::DBConn>,
}

const REGEX_URL_EXTRACTOR: &str = r"\b(?:https?:\/\/|<)[^\s>]+(?:>|)\b";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        let url: String;
        let content = msg.content.clone();
        let options: Vec<CreateSelectMenuOption>;

        let id = if msg.guild_id.is_some() {
            msg.guild_id.unwrap().0
        } else {
            msg.author.id.0
        };

        let get_lang = match self.dbconn.lock().await.get_server(id, false).language {
            Some(lang) => lang,
            None => "en".to_string(),
        };
        let lang = get_lang.as_str();

        if twitter::is_twitter_url(content.as_str())
            && self.dbconn.lock().await.get_server(id, false).twitter
        {
            url = twitter::remove_tracking(
                twitter::convert_url_lazy(content, twitter::UrlType::Vxtwitter).await,
            );

            options = get_twitter_options(lang);
        } else if bluesky::is_bluesky_url(content.as_str())
            && self.dbconn.lock().await.get_server(id, false).bluesky
        {
            url = bluesky::convert_url_lazy(content, bluesky::UrlType::FixBluesky).await;
            options = get_blueksy_options(lang);
        } else if tiktok::is_tiktok_url(content.as_str())
            && self.dbconn.lock().await.get_server(id, false).tiktok
        {
            url =
                tiktok::convert_url_lazy(tiktok::clear_url(content).await, tiktok::UrlType::TIKTXK)
                    .await;
            options = get_tik_tok_options(lang);
        } else if instagram::is_instagram_url(content.as_str())
            && self.dbconn.lock().await.get_server(id, false).instagram
        {
            url = instagram::convert_url_lazy(content, instagram::UrlType::DDInstagram).await;
            options = get_instagram_options(lang);
        } else if msg.referenced_message.is_some() {
            let ref_message = &msg.referenced_message.clone().unwrap();
            if ref_message.author.id != context.http.get_current_user().await.unwrap().id {
                return;
            };

            let mut ref_author = ref_message
                .content
                .split_once(':')
                .unwrap()
                .0
                .split_at(2)
                .1
                .to_string();
            ref_author.pop();

            let author = &context
                .http
                .get_user(ref_author.parse::<u64>().unwrap())
                .await
                .unwrap();

            // Ignore people that reply to their own messages
            if author.id == msg.author.id {
                return;
            }

            let msg_url = &msg.link_ensured(&context.http).await;
            let author_nickname = &msg
                .author
                .nick_in(&context.http, &msg.guild_id.unwrap())
                .await
                .unwrap_or(msg.author.name.clone());
            author
                .dm(&context.http, |m| {
                    m.content(t!(
                        "referenced", locale=lang,
                        USER_ID = &msg.author.id,
                        AUTHOR_NICKNAME = &author_nickname,
                        MESSAGE_URL = &msg_url
                    ))
                })
                .await
                .unwrap();

            return;
        } else {
            return;
        }

        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(": ")
            .push(url)
            .build();

        if let Err(why) = msg
            .channel_id
            .send_message(&context.http, |m| {
                m.allowed_mentions(|am| am.empty_parse()).content(response);
                if msg.referenced_message.is_some() {
                    m.reference_message(msg.message_reference.clone().unwrap());
                }
                m.components(|f| {
                    f.create_action_row(|f| {
                        f.create_select_menu(|s| {
                            s.custom_id("select")
                                .placeholder(t!("nothing_selected", locale=lang))
                                .min_values(1)
                                .max_values(1)
                                .options(|o| o.set_options(options))
                        })
                    })
                })
            })
            .await
        {
            println!("{}", t!("error_sending_message", locale=lang, WHY = why));
        };

        if !msg.is_private() {
            // Delete message
            if let Err(why) = msg.delete(&context.http).await {
                println!("{}", t!("error_delete_message", locale=lang, WHY = why));
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // Check whether button has been pressed
        if interaction.kind() != InteractionType::MessageComponent {
            return;
        }

        let component = interaction.as_message_component().unwrap().clone();
        let command = component.data.values.first().unwrap();
        let msg = &component.message;

        if !msg.author.bot {
            return;
        }

        // Get guild ID
        let id =  msg.author.id.0;
        let get_lang = match self.dbconn.lock().await.get_server(id, false).language {
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
                t!("source_code", locale=lang, URL = "https://github.com/AnnsAnns/sphene").to_string()
            } else if command == "menu" {
                t!("menu_meme").to_string()
            } else if command == "disable" {
                t!("disable").to_string()
            } else if command == "set_language" || command == "contribute_language"{
                t!("contribute_language", locale=lang, URL="https://github.com/AnnsAnns/sphene/locales").to_string()
            } else if command == "download" {
                let extracted_url = self
                    .regex_pattern
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
                    t!("download_url", locale=lang, URL = url).to_string()
                } else {
                    t!("no_download", locale=lang).to_string()
                }
            } else {
                t!("not_author", locale=lang).to_string()
            };

            component
                .create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m.content(content).ephemeral(true))
                })
                .await
                .unwrap();

            return;
        } else {
            // Make the Discord API happy :)
            component
                .create_interaction_response(&ctx.http, |r| {
                    r.kind(InteractionResponseType::DeferredUpdateMessage)
                })
                .await
                .unwrap();
        }

        if command == "remove" {
            if let Err(why) = component
                .edit_original_interaction_response(&ctx.http, |m| {
                    m.content(t!("deleted_message", locale=lang))
                        .allowed_mentions(|am| am.empty_parse());
                    m.components(|c| c)
                })
                .await
            {
                println!("Error editing message: {:?}", why);
            }

            // Sleep for 5 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // Delete the response message
            if let Err(why) = component
                .delete_original_interaction_response(&ctx.http)
                .await
            {
                println!("Error deleting message: {:?}", why);
            }
        } else {
            let extracted_url = self
                .regex_pattern
                .find_iter(&msg.content)
                .next()
                .unwrap()
                .as_str()
                .to_string();
            let mut new_msg: String = String::new();

            let mut twitter_urltype = twitter::UrlType::from_string(command);
            let bluesky_urltype = bluesky::UrlType::from_string(command);
            let instagram_urltype = instagram::UrlType::from_string(command);
            let tiktok_urltype = tiktok::UrlType::from_string(command);

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

            if let Err(why) = component
                .edit_original_interaction_response(&ctx.http, |m| {
                    m.content(new_msg).allowed_mentions(|am| am.empty_parse())
                })
                .await
            {
                println!("Error editing message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::watching("out for embeds 🕵️"))
            .await;

        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file.
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let regex_pattern = Regex::new(REGEX_URL_EXTRACTOR).unwrap();

    let dbconn = Mutex::new(DBConn::new().unwrap());

    println!("Available Languages: {:?}", available_locales!());

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            regex_pattern,
            dbconn,
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
