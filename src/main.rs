use std::env;

extern crate dotenv;

use dotenv::dotenv;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Reaction;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Handler;

const TWITTER_URL: &str = "https://twitter.com/";
const FXTWITTER_URL: &str = "https://fxtwitter.com/";
const VXTWITTER_URL: &str = "https://vxtwitter.com/";
const REMOVE_REACTION: char = '❌';
const SWITCH_REACTION: char = '🔄';

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if !msg.content.contains(TWITTER_URL) {
            return;
        }

        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(": ")
            .push(msg.content.replace(TWITTER_URL, FXTWITTER_URL))
            .build();

        let rsp_msg = msg
            .channel_id
            .send_message(&context.http, |m| {
                m.allowed_mentions(|am| am.empty_parse()).content(response)
            })
            .await;

        if let Err(why) = &rsp_msg {
            println!("Error sending message: {:?}", why);
        }
        let rsp_msg = rsp_msg.unwrap();

        // Add remove reaction to the message
        if let Err(why) = &rsp_msg.react(&context.http, REMOVE_REACTION).await {
            println!("Error adding remove reaction: {:?}", why);
        }

        // Add switch reaction to the message
        if let Err(why) = &rsp_msg.react(&context.http, SWITCH_REACTION).await {
            println!("Error adding switching reaction: {:?}", why);
        }

        // Delete message
        if let Err(why) = msg.delete(&context.http).await {
            println!("Error deleting message: {:?}", why);
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if !(add_reaction.emoji.unicode_eq(&REMOVE_REACTION.to_string())
            || add_reaction.emoji.unicode_eq(&SWITCH_REACTION.to_string()))
        {
            return;
        }

        let mut msg = add_reaction.message(&ctx.http).await.unwrap();
        // If the message is not from the bot return
        if !msg.author.bot {
            return;
        }

        // Get the user who added the reaction
        let user = add_reaction
            .user_id
            .unwrap()
            .to_user(&ctx.http)
            .await
            .unwrap();
        // If the user is the bot return
        if user.bot {
            return;
        }

        // Check whether user is correct
        if !msg.content.contains(&user.id.to_string()) {
            return;
        }

        // If not the remove reaction return
        if add_reaction.emoji.unicode_eq(&REMOVE_REACTION.to_string()) {
            if let Err(why) = msg.delete(&ctx.http).await {
                println!("Error deleting message: {:?}", why);
            }

            // Deleted Message Response
            let rsp_msg = msg.channel_id.say(&ctx.http, "💣 Deleted Message").await;
            if let Err(why) = &rsp_msg {
                println!("Error sending message: {:?}", why);
            }

            // Sleep for 5 seconds
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // Delete the response message
            if let Err(why) = rsp_msg.unwrap().delete(&ctx.http).await {
                println!("Error deleting message: {:?}", why);
            }
        } else {
            let mut new_msg = msg.content.clone();

            if new_msg.contains(FXTWITTER_URL) {
                new_msg = new_msg.replace(FXTWITTER_URL, VXTWITTER_URL);
            } else {
                new_msg = new_msg.replace(VXTWITTER_URL, FXTWITTER_URL);
            }

            // This is required to fix a potential caching issue with embeds
            msg.embeds.clear();
            
            if let Err(why) = msg.edit(&ctx.http, |m| m.content(new_msg)).await {
                println!("Error changing message: {:?}", why);
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
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
