use crate::{utils::{get_relevant_id, parse_choice, Choices, EnableOrDisable}, Context, Error};

#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn change(
    ctx: Context<'_>,
    #[description = "What do you want to change?"] choice: Choices,
    #[description = "Should it be enabled or disabled?"] enable_or_disable: EnableOrDisable,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let id = get_relevant_id(ctx);
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
