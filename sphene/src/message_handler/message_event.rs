use poise::serenity_prelude::{
    Context, CreateActionRow, CreateAllowedMentions, CreateMessage,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, FullEvent, Message,
    MessageBuilder, UserId,
};
use rust_i18n::t;
use thorium::{bluesky, db::DBConn, instagram, tiktok, twitter};
use tokio::sync::Mutex;

use crate::{
    options::{
        get_blueksy_options, get_instagram_options, get_tik_tok_options, get_twitter_options,
    },
    Data,
};

pub async fn message(context: &Context, msg: Message, dbconn: &Mutex<DBConn>) {
    let url: String;
    let content = msg.content.clone();
    let options: Vec<CreateSelectMenuOption>;

    let id = if msg.guild_id.is_some() {
        msg.guild_id.unwrap().get()
    } else {
        msg.author.id.get()
    };

    let get_lang = match dbconn.lock().await.get_server(id, false).language {
        Some(lang) => lang,
        None => "en".to_string(),
    };
    let lang = get_lang.as_str();

    if twitter::is_twitter_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).twitter
    {
        url = twitter::remove_tracking(
            twitter::convert_url_lazy(content, twitter::UrlType::Vxtwitter).await,
        );

        options = get_twitter_options(lang);
    } else if bluesky::is_bluesky_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).bluesky
    {
        url = bluesky::convert_url_lazy(content, bluesky::UrlType::FixBluesky).await;
        options = get_blueksy_options(lang);
    } else if tiktok::is_tiktok_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).tiktok
    {
        url = tiktok::convert_url_lazy(tiktok::clear_url(content).await, tiktok::UrlType::TIKTXK)
            .await;
        options = get_tik_tok_options(lang);
    } else if instagram::is_instagram_url(content.as_str())
        && dbconn.lock().await.get_server(id, false).instagram
    {
        url = instagram::convert_url_lazy(content, instagram::UrlType::DDInstagram).await;
        options = get_instagram_options(lang);
    } else if msg.referenced_message.is_some() {
        let ref_message = &msg.referenced_message.clone().unwrap();
        if ref_message.author.id != context.http.get_current_user().await.unwrap().id {
            return;
        };

        let mut ref_author = ref_message
            .content
            .split_once(':')
            .unwrap()
            .0
            .split_at(2)
            .1
            .to_string();
        ref_author.pop();

        let user_id = UserId::new(ref_author.parse::<u64>().unwrap());

        let author = &context.http.get_user(user_id).await.unwrap();

        // Ignore people that reply to their own messages
        if author.id == msg.author.id {
            return;
        }

        let msg_url = &msg.link_ensured(&context.http).await;
        let author_nickname = &msg
            .author
            .nick_in(&context.http, &msg.guild_id.unwrap())
            .await
            .unwrap_or(msg.author.name.clone());

        let message = CreateMessage::new().content(t!(
            "referenced",
            locale = lang,
            USER_ID = &msg.author.id,
            AUTHOR_NICKNAME = &author_nickname,
            MESSAGE_URL = &msg_url
        ));

        author.dm(&context.http, message).await.unwrap();

        return;
    } else {
        return;
    }

    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(": ")
        .push(url)
        .build();

    let allowedMentions = CreateAllowedMentions::new().empty_users().empty_roles();

    let mut message = CreateMessage::new()
        .allowed_mentions(allowedMentions)
        .content(response);

    if msg.referenced_message.is_some() {
        message = message.reference_message(msg.message_reference.clone().unwrap());
    };

    let selectMenu =
        CreateSelectMenu::new("select", CreateSelectMenuKind::String { options: options })
            .max_values(1)
            .min_values(1)
            .placeholder(t!("nothing_selected", locale = lang));

    let actionRow = CreateActionRow::SelectMenu(selectMenu);

    message = message.components(vec![actionRow]);

    if let Err(why) = msg.channel_id.send_message(&context.http, message).await {
        println!("{}", t!("error_sending_message", locale = lang, WHY = why));
    };

    if !msg.is_private() {
        // Delete message
        if let Err(why) = msg.delete(&context.http).await {
            println!("{}", t!("error_delete_message", locale = lang, WHY = why));
        }
    }
}
