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
use serenity::model::prelude::UserId;
use serenity::prelude::*;
use thorium::twitter::UrlType;

struct Handler {
    channel_id: ChannelId,
    user_id: UserId,
}

async fn process_tweet(
    url: String,
    channel_id: ChannelId,
    msg: Message,
    context: Context,
    target_channel: ChannelId,
) {
    if let Err(why) = channel_id
        .send_message(&context.http, |m| {
            m.allowed_mentions(|am| am.empty_parse());
            if url != "0" {
                m.add_file(AttachmentType::Image(Url::parse(&url).unwrap()));
            }
            if msg.referenced_message.is_some() {
                m.reference_message(msg.message_reference.clone().unwrap());
            }
            if url == "0" {
                m.content(msg.content.clone());
            } else if channel_id == target_channel {
                m.content(format!("<{}>", msg.content.clone()));
            }
            m.components(|f| {
                f.create_action_row(|f| {
                    f.create_button(|b| {
                        b.custom_id("remove")
                            .label("Remove")
                            .style(ButtonStyle::Secondary)
                    });
                    if url != "0" {
                        f.create_button(|b| {
                            b.custom_id("switch")
                                .label("Switch")
                                .style(ButtonStyle::Secondary)
                        });
                    }
                    f.create_button(|b| {
                        b.0.insert("url", Value::from(msg.content.to_string()));
                        b.style(ButtonStyle::Link).label("Source")
                    })
                })
            })
        })
        .await
    {
        println!("Error sending message: {:?}", why);
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.author.id != self.user_id
            || msg.author.bot
            || !thorium::bluesky::is_bluesky_url(msg.content.as_str())
        {
            return;
        }

        let url: String;

        if thorium::twitter::is_twitter_url(msg.content.as_str()) {
            url = thorium::twitter::get_media_from_url(
                thorium::twitter::convert_url_lazy(msg.content.clone(), UrlType::Vxtwitter).await,
            )
            .await;
        } else if thorium::bluesky::is_bluesky_url(msg.content.as_str()) {
            url = thorium::bluesky::get_media_from_url(msg.content.clone()).await;
        } else {
            return;
        }

        let channel_id = if msg.is_private() {
            self.channel_id
        } else {
            msg.channel_id
        };

        process_tweet(
            url,
            channel_id,
            msg.clone(),
            context.clone(),
            self.channel_id,
        )
        .await;

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

        if custom_id == "remove" {
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
        } else if custom_id == "switch" {
            // Delete the response message
            if let Err(why) = component
                .delete_original_interaction_response(&ctx.http)
                .await
            {
                println!("Error deleting message: {:?}", why);
            }

            process_tweet(
                thorium::twitter::convert_url_lazy(msg.content.clone(), UrlType::Vxtwitter).await,
                self.channel_id,
                msg.clone(),
                ctx,
                component.channel_id,
            )
            .await;
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
    let user_id = env::var("USER_ID").expect("Expected a user id in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            channel_id: ChannelId::from(channel_id.parse::<u64>().unwrap()),
            user_id: UserId::from(user_id.parse::<u64>().unwrap()),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
