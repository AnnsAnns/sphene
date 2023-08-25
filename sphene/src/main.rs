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
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use thorium::*;

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

        // Make the Discord API happy no matter what :)
        component
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
            .unwrap();

        let msg = &component.message;
        if !msg.author.bot {
            return;
        }

        let user = &component.user.id.to_string();
        // Check whether user is correct
        if !msg.content.contains(user) {
            return;
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
                new_msg =
                    bluesky::convert_url_lazy(extracted_url.to_string(), bluesky::UrlType::FixBluesky).await;
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

    async fn ready(&self, _: Context, ready: Ready) {
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

    let twitter_options: Vec<CreateSelectMenuOption> = vec![
        CreateSelectMenuOption::new("Menu", "None")
            .default_selection(true)
            .to_owned(),
        CreateSelectMenuOption::new("🔄️ Change to: VXTwitter", twitter::VXTWITTER_URL).to_owned(),
        CreateSelectMenuOption::new("🔄️ Change to: FXTwitter", twitter::FXTWITTER_URL).to_owned(),
        CreateSelectMenuOption::new("🖼️ Image Only: VXTwitter", "direct_vx").to_owned(),
        CreateSelectMenuOption::new("🖼️ Image Only: FXTwitter", "direct_fx").to_owned(),
        CreateSelectMenuOption::new("🤨 Show original Twitter URL", twitter::TWITTER_URL)
            .to_owned(),
        CreateSelectMenuOption::new("❌ Remove this Message", "remove").to_owned(),
    ];

    let bluesky_options: Vec<CreateSelectMenuOption> = vec![
        CreateSelectMenuOption::new("Menu", "None")
            .default_selection(true)
            .to_owned(),
        CreateSelectMenuOption::new("🔄️ Change to: Psky", bluesky::PSKY_URL).to_owned(),
        CreateSelectMenuOption::new("🔄️ Change to: FixBluesky", bluesky::FIXBLUESKY_URL).to_owned(),
        CreateSelectMenuOption::new("🖼️ Image Only", "direct_fxbsky").to_owned(),
        CreateSelectMenuOption::new("☁️ Show original Bluesky URL", bluesky::BLUESKY_URL).to_owned(),
        CreateSelectMenuOption::new("❌ Remove this Message", "remove").to_owned(),
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
