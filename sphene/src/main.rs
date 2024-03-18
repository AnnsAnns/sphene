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

#[derive(Debug, Clone, poise::ChoiceParameter)]
pub enum EnableOrDisable {
    Enable = 1,
    Disable = 0,
}

#[derive(Debug, Clone, poise::ChoiceParameter)]
pub enum Languages {
    English,
    German,
    Dutch,
}

// Allow enum to string conversion for Languages
trait ToLanguageString {
    fn to_language_string(&self) -> String;
}

impl ToLanguageString for Languages {
    fn to_language_string(&self) -> String {
        match self {
            Languages::English => "en".to_string(),
            Languages::German => "de-DE".to_string(),
            Languages::Dutch => "nl-NL".to_string(),
        }
    }
}

fn parse_choice(choice: &Choices, mut server: Server, change_to: bool) -> Server {
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

#[poise::command(slash_command, prefix_command)]
async fn set_own_language(
    ctx: Context<'_>,
    #[description = "Which language should the bot use for you personally?"] language: Languages,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let id = ctx.author().id.get();
    let mut server = db.get_server(id, true);
    server.language = Some(language.to_language_string());
    db.update_server(server);
    ctx.say(format!("Changed language to {:#?} 👍", language))
        .await?;
    Ok(())
}

// Gets relevent id, if guild id is present, it returns the guild id, else it returns the author id
async fn get_relevant_id(ctx: Context<'_>) -> u64 {
    if ctx.guild_id().is_some() {
        ctx.guild_id().unwrap().get()
    } else {
        ctx.author().id.get()
    }
}

#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn set_guild_language(
    ctx: Context<'_>,
    #[description = "Which language should the bot use for this Guild?"] language: Languages,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let id = get_relevant_id(ctx).await;
    let mut server = db.get_server(id, true);
    server.language = Some(language.to_language_string());
    db.update_server(server);
    ctx.say(format!("Changed language to {:#?} 👍", language))
        .await?;
    Ok(())
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
async fn change(
    ctx: Context<'_>,
    #[description = "What do you want to change?"] choice: Choices,
    #[description = "Should it be enabled or disabled?"] enable_or_disable: EnableOrDisable,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let id = get_relevant_id(ctx).await;
    let mut server = db.get_server(id, true);
    server = parse_choice(&choice, server, enable_or_disable.clone() as u8 == 1);
    db.update_server(server);
    ctx.say(format!(
        "Changed {:#?} to {:#?}d 👍",
        choice, enable_or_disable
    ))
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let dbconn = Mutex::new(DBConn::new().unwrap());
    dbconn.lock().await.create_new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![change, set_own_language, set_guild_language],
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
