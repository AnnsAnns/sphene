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
const REMOVE_REACTION: char = '❌';

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

        let rsp_msg = msg.channel_id.send_message(
            &context.http, 
            |m| {
                m.allowed_mentions(|am| am.empty_parse())
                .content(response)
            }
        ).await;

        if let Err(why) = &rsp_msg {
            println!("Error sending message: {:?}", why);
        }

        // Add reaction to the message
        if let Err(why) = rsp_msg.unwrap().react(&context.http, REMOVE_REACTION).await {
            println!("Error adding reaction: {:?}", why);
        }

        // Delete message
        if let Err(why) = msg.delete(&context.http).await {
            println!("Error deleting message: {:?}", why);
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        // If not the remove reaction return
        if !add_reaction.emoji.unicode_eq(&REMOVE_REACTION.to_string()) {
            return;
        }

        let msg = add_reaction.message(&ctx.http).await.unwrap();
        // If the message is not from the bot return
        if !msg.author.bot {
            return;
        }

        // Get the user who added the reaction
        let user = add_reaction.user_id.unwrap().to_user(&ctx.http).await.unwrap();
        // If the user is the bot return
        if user.bot {
            return;
        }

        // If the user is the person mentioned in the message delete the message
        if !msg.content.contains(&user.id.to_string()) {
            return;
        }

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
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}