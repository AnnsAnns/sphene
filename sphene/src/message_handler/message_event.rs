

use poise::serenity_prelude::{
    Context, CreateActionRow, CreateAllowedMentions, CreateMessage,
    CreateSelectMenu, CreateSelectMenuKind, Message,
    MessageBuilder, UserId,
};
use rust_i18n::t;
use thorium::{db::DBConn};
use tokio::sync::Mutex;
use rand::seq::{IndexedRandom, SliceRandom};

use crate::{
    commands::convert_url::convert_url
};

pub async fn message(context: &Context, msg: Message, dbconn: &Mutex<DBConn>) {
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

    let converted_url = convert_url(msg.clone(), dbconn, id, lang).await;

    if converted_url.is_none() && msg.referenced_message.is_none() {
        return;
    } else if converted_url.is_none() && msg.referenced_message.is_some() {
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
    }

    let converted_url = converted_url.unwrap();
    let url = converted_url.url;
    let options = converted_url.options;

    const NEWS_URLS: [&str; 5] = [
        "https://www.nbcnews.com/news/world/elon-musks-call-germany-move-nazi-guilt-dangerous-holocaust-memorial-c-rcna189316",
        "https://en.wikipedia.org/wiki/Views_of_Elon_Musk#Race_and_white_nationalism",
        "https://en.wikipedia.org/wiki/Political_activities_of_Elon_Musk",
        "https://www.nytimes.com/2025/03/14/technology/elon-musk-x-post-hitler-stalin-mao.html",
        "https://cdn.bsky.app/img/feed_thumbnail/plain/did:plc:y5xyloyy7s4a2bwfeimj7r3b/bafkreihuamazhsgewbxcg5fyqw4akvjaspyqz7cmhpdhyztg7fon5t7v5y@jpeg"
    ];

    let article = NEWS_URLS.choose(&mut rand::rng()).unwrap().to_string();

    let extra_info = if url.contains("twitter.com") {
        // Include news link
        format!("\n Stop using Twitter: {} \n Twitter support will stop functioning on the 1st of April! \n You can disable Twitter support early using `/change choice:Twitter enable_or_disable:Disable`.", article)
    } else {
        "".to_string()
    };

    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(": ")
        .push(url)
        .push(extra_info)
        .build();

    let allowed_mentions = CreateAllowedMentions::new().empty_users().empty_roles();

    let mut message = CreateMessage::new()
        .allowed_mentions(allowed_mentions)
        .content(response);

    if msg.referenced_message.is_some() {
        message = message.reference_message(msg.message_reference.clone().unwrap());
    };

    let select_menu =
        CreateSelectMenu::new("select", CreateSelectMenuKind::String { options })
            .max_values(1)
            .min_values(1)
            .placeholder(t!("nothing_selected", locale = lang));

    let action_row = CreateActionRow::SelectMenu(select_menu);

    message = message.components(vec![action_row]);

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
