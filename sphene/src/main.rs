use std::env;

extern crate dotenv;

use dotenv::dotenv;

use regex::Regex;
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
use thorium::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Handler {
    options_twitter: Vec<CreateSelectMenuOption>,
    options_bluesky: Vec<CreateSelectMenuOption>,
    regex_pattern: Regex,
}

const REGEX_URL_EXTRACTOR: &str = r"\b(?:https?:\/\/|<)[^\s>]+(?:>|)\b";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        let url: String;
        let content = msg.content.clone();
        let options: Vec<CreateSelectMenuOption>;

        if twitter::is_twitter_url(content.as_str()) {
            url = twitter::convert_url_lazy(content, twitter::UrlType::Vxtwitter).await;
            options = self.options_twitter.clone();
        } else if bluesky::is_bluesky_url(content.as_str()) {
            url = bluesky::convert_url_lazy(content, bluesky::UrlType::FixBluesky).await;
            options = self.options_bluesky.clone();
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

            println!("ref_author: {}", &ref_author);
            let author = &context
                .http
                .get_user(ref_author.parse::<u64>().unwrap())
                .await
                .unwrap();

            let msg_url = &msg.link_ensured(&context.http).await;
            let author_nickname = &msg
                .author
                .nick_in(&context.http, &msg.guild_id.unwrap())
                .await
                .unwrap_or(msg.author.name.clone());
            author
                .dm(&context.http, |m| {
                    m.content(format!(
                        "🔗 Your Message has been referenced by <@{}> ({}) in: {}",
                        &msg.author.id, &author_nickname, &msg_url
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
                                .placeholder("Nothing selected")
                                .min_values(1)
                                .max_values(1)
                                .options(|o| o.set_options(options))
                        })
                    })
                })
            })
            .await
        {
            println!("Error sending message: {:?}", why);
        };

        if !msg.is_private() {
            // Delete message
            if let Err(why) = msg.delete(&context.http).await {
                println!("Error deleting message: {:?}", why);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        // Check whether button has been pressed
        if interaction.kind() != InteractionType::MessageComponent {
            return;
        }

        let component = interaction.as_message_component().unwrap().clone();
        let command = component.data.values.get(0).unwrap();
        let msg = &component.message;

        if !msg.author.bot {
            return;
        }

        let user = &component.user.id.to_string();
        // Check whether user is correct
        if !msg.content.contains(user)
            || command == "version"
            || command == "download"
            || command == "menu"
        {
            let content = if command == "version" {
                "☁️ The Source Code can be found at: https://github.com/AnnsAnna/sphene".to_string()
            } else if command == "menu" {
                "🕺 https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()
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
                } else {
                    bluesky::get_media_from_url(
                        bluesky::convert_url_lazy(extracted_url, bluesky::UrlType::FixBluesky)
                            .await,
                    )
                    .await
                };

                if url != "0" {
                    format!("⏬ Your Download URL is: <{}>", url)
                } else {
                    "⚠️ No Download URL found!".to_string()
                }
            } else {
                "⚠️ You are not the author of this message!".to_string()
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
                    m.content("💣 Deleted Message")
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

            if twitter_urltype != twitter::UrlType::Unknown {
                new_msg = twitter::convert_url_lazy(extracted_url, twitter_urltype).await;
            } else if bluesky_urltype != bluesky::UrlType::Unknown {
                new_msg = bluesky::convert_url_lazy(extracted_url, bluesky_urltype).await;
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

    let default_option = CreateSelectMenuOption::new("Menu", "menu")
        .default_selection(true)
        .to_owned();
    let remove_option = CreateSelectMenuOption::new("❌ Remove this Message", "remove").to_owned();
    let version_option = CreateSelectMenuOption::new(
        format!("🏳️‍⚧️ Running v{} of Sphene using Thorium", VERSION),
        "version",
    )
    .to_owned();
    let download_option = CreateSelectMenuOption::new("⏬ Download Media", "download").to_owned();

    let twitter_options: Vec<CreateSelectMenuOption> = vec![
        download_option.clone(),
        CreateSelectMenuOption::new("🔄️ Change to: VXTwitter", twitter::VXTWITTER_URL).to_owned(),
        CreateSelectMenuOption::new("🔄️ Change to: FXTwitter", twitter::FXTWITTER_URL).to_owned(),
        CreateSelectMenuOption::new("🖼️ Media Only: VXTwitter", "direct_vx").to_owned(),
        CreateSelectMenuOption::new("🖼️ Media Only: FXTwitter", "direct_fx").to_owned(),
        CreateSelectMenuOption::new("🤨 Show original Twitter URL", twitter::TWITTER_URL)
            .to_owned(),
        remove_option.clone(),
        version_option.clone(),
        default_option.clone(),
    ];

    let bluesky_options: Vec<CreateSelectMenuOption> = vec![
        download_option,
        CreateSelectMenuOption::new("🔄️ Change to: Psky", bluesky::PSKY_URL).to_owned(),
        CreateSelectMenuOption::new("🔄️ Change to: FixBluesky", bluesky::FIXBLUESKY_URL).to_owned(),
        CreateSelectMenuOption::new("🖼️ Media Only", "direct_fxbsky").to_owned(),
        CreateSelectMenuOption::new("☁️ Show original Bluesky URL", bluesky::BLUESKY_URL).to_owned(),
        remove_option,
        version_option,
        default_option,
    ];

    let regex_pattern = Regex::new(REGEX_URL_EXTRACTOR).unwrap();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            options_twitter: twitter_options,
            options_bluesky: bluesky_options,
            regex_pattern,
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
