use std::env;

extern crate dotenv;

use dotenv::dotenv;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, mut msg: Message) {
        if msg.content.contains("https://twitter.com/") {
            let response = MessageBuilder::new()
                .mention(&msg.author)
                .push(": ")
                .push(msg.content.replace("https://twitter.com/", "https://fxtwitter.com/"))
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }

            if msg.content.find(" ").is_none() {
                // Delete message
                if let Err(why) = msg.delete(&context.http).await {
                    println!("Error deleting message: {:?}", why);
                }
            } else {
                // Suppress embed
                if let Err(why) = msg.suppress_embeds(&context.http).await {
                    println!("Error removing embed message: {:?}", why);
                }
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
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}