use crate::{utils::{get_relevant_id, Languages, ToLanguageString}, Context, Error};

#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn set_guild_language(
    ctx: Context<'_>,
    #[description = "Which language should the bot use for this Guild?"] language: Languages,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let id = get_relevant_id(ctx);
    let mut server = db.get_server(id, true);
    server.language = Some(language.to_language_string());
    db.update_server(server);
    ctx.say(format!("Changed language to {:#?} 👍", language))
        .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn set_own_language(
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