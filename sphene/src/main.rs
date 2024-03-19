use poise::serenity_prelude as serenity;
use thorium::db::{DBConn};
use tokio::sync::Mutex;
use commands::set_lang::{set_guild_language, set_own_language};
use commands::change::change;



rust_i18n::i18n!("../locales", fallback = "en");

struct Data {
    db: Mutex<DBConn>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

mod commands;
mod utils;
mod message_handler;
mod options;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let dbconn = Mutex::new(DBConn::new().unwrap());
    dbconn.lock().await.create_new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![change(), set_own_language(), set_guild_language()],
            event_handler: |ctx, event, framework, data| { 
                Box::pin(message_handler::event_handler(ctx, event, framework, data))   
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db: dbconn })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
