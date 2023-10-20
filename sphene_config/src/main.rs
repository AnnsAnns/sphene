use poise::serenity_prelude as serenity;
use thorium::db::{DBConn, Server};
use tokio::sync::Mutex;

struct Data {
    db: Mutex<DBConn>,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, poise::ChoiceParameter)]
pub enum Choices {
    Twitter,
    Bluesky,
    Instagram,
    Tiktok,
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum EnableOrDisable {
    Enable = 1,
    Disable = 0,
}

fn parse_choice(choice: Choices, mut server: Server, change_to: bool) -> Server {
    match choice {
        Choices::Twitter => {
            server.twitter = change_to;
        }
        Choices::Bluesky => {
            server.bluesky = change_to;
        }
        Choices::Instagram => {
            server.instagram = change_to;
        }
        Choices::Tiktok => {
            server.tiktok = change_to;
        }
    }
    server
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn change(
    ctx: Context<'_>,
    #[description = "What do you want to change?"] choice: Choices,
    #[description = "Should it be enabled or disabled?"] enable_or_disable: EnableOrDisable,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let id = if ctx.guild_id().is_some() {
        ctx.guild_id().unwrap().0
    } else {
        ctx.author().id.0
    };
    let mut server = db.get_server(id, true);
    server = parse_choice(choice, server, enable_or_disable as u8 == 1);
    db.update_server(server);
    ctx.say("Changed 👍").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let dbconn = Mutex::new(DBConn::new().unwrap());
    dbconn.lock().await.create_new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![change()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { db: dbconn })
            })
        });

    framework.run().await.unwrap();
}
