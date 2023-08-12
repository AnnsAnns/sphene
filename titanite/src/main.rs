use std::env;

extern crate dotenv;

use dotenv::dotenv;

use reqwest::Url;
use serenity::async_trait;
use serenity::json::Value;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::Interaction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::InteractionType;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::AttachmentType;
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;

struct Handler {
    channel_id: ChannelId,
}

const TWITTER_URL: &str = "https://twitter.com/";
const X_URL: &str = "https://x.com/";
const FXTWITTER_URL: &str = "https://d.fxtwitter.com/";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if !(msg.content.contains(TWITTER_URL) || msg.content.contains(X_URL)) {
            return;
        }

        let mut url = String::from(msg.content.clone());
        if url.contains(TWITTER_URL) {
            url = url.replace(TWITTER_URL, FXTWITTER_URL);
        } else {
            url = url.replace(X_URL, FXTWITTER_URL);
        }

        if !url.contains(".mp4") {
            url.push_str(".jpg");
        }

        let channel_id = if msg.is_private() { self.channel_id } else { msg.channel_id };

        if let Err(why) = channel_id
            .send_message(&context.http, |m| {
                m.allowed_mentions(|am| am.empty_parse())
                    .add_file(AttachmentType::Image(Url::parse(&url).unwrap()));
                if msg.referenced_message.is_some() {
                    m.reference_message(msg.message_reference.clone().unwrap());
                }
                if channel_id == self.channel_id {
                    m.content(format!("<{}>", msg.content.clone()));
                }
                m.components(|f| {
                    f.create_action_row(|f| {
                        f.create_button(|b| {
                            b.custom_id("remove")
                                .label("Remove")
                                .style(ButtonStyle::Secondary)
                        });
                        f.create_button(|b| {
                            b.0.insert("url", Value::from(msg.content.to_string()));
                            b.style(ButtonStyle::Link)
                                .label("Source")
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
        let custom_id = component.data.custom_id.to_string();

        if custom_id != "remove" {
            return;
        }

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
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Delete the response message
        if let Err(why) = component
            .delete_original_interaction_response(&ctx.http)
            .await
        {
            println!("Error deleting message: {:?}", why);
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
    let channel_id = env::var("CHANNEL_ID").expect("Expected a channel id in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            channel_id: ChannelId::from(channel_id.parse::<u64>().unwrap()),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
