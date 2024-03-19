use poise::serenity_prelude::CreateSelectMenuOption;

use rust_i18n::t;
use thorium::{bluesky, instagram, tiktok, twitter};

const VERSION: &str = env!("CARGO_PKG_VERSION");

rust_i18n::i18n!("locales", fallback = "en");

pub fn get_remove_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("remove", locale = lang), "remove")
}

pub fn get_disable_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("change_info", locale = lang), "disable")
}

pub fn get_default_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("menu", locale = lang), "menu")
        .default_selection(true)
        .to_owned()
}

pub fn get_set_language_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("change_language", locale = lang), "set_language")
}

pub fn get_contribute_language_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("contribute_languages_option", locale = lang), "contribute_language")
}

pub fn get_download_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("download", locale = lang), "download")
}

pub fn get_version_option(lang: &str) -> CreateSelectMenuOption {
    CreateSelectMenuOption::new(t!("version", locale = lang, VERSION = VERSION), "version")
}

pub fn get_blueksy_options(lang: &str) -> Vec<CreateSelectMenuOption> {
    vec![
        get_download_option(lang),
        CreateSelectMenuOption::new(t!("psky", locale = lang), bluesky::PSKY_URL),
        CreateSelectMenuOption::new(t!("fixbluesky", locale = lang), bluesky::FIXBLUESKY_URL),
        CreateSelectMenuOption::new(t!("media_only", locale = lang), "direct_fxbsky"),
        CreateSelectMenuOption::new(
            t!("show_original_bluesky", locale = lang),
            bluesky::BLUESKY_URL,
        ),
        get_remove_option(lang),
        get_set_language_option(lang),
        get_contribute_language_option(lang),
        get_disable_option(lang),
        get_version_option(lang),
        get_default_option(lang),
    ]
}

pub fn get_twitter_options(lang: &str) -> Vec<CreateSelectMenuOption> {
    vec![
        get_download_option(lang),
        CreateSelectMenuOption::new(t!("vxtwitter", locale = lang), twitter::VXTWITTER_URL),
        CreateSelectMenuOption::new(t!("fxtwitter", locale = lang), twitter::FXTWITTER_URL),
        CreateSelectMenuOption::new(t!("media_only_vxtwitter", locale = lang), "direct_vx"),
        CreateSelectMenuOption::new(t!("media_only_fxtwitter", locale = lang), "direct_fx"),
        CreateSelectMenuOption::new(
            t!("show_original_twitter", locale = lang),
            twitter::TWITTER_URL,
        ),
        get_remove_option(lang),
        get_set_language_option(lang),
        get_contribute_language_option(lang),
        get_disable_option(lang),
        get_version_option(lang),
        get_default_option(lang),
    ]
}

pub fn get_instagram_options(lang: &str) -> Vec<CreateSelectMenuOption> {
    vec![
        CreateSelectMenuOption::new(t!("ddinstagram", locale = lang), instagram::DDINSTAGRAM_URL),
        CreateSelectMenuOption::new(
            t!("show_original_instagram", locale = lang),
            instagram::INSTAGRAM_URL,
        ),
        get_remove_option(lang),
        get_set_language_option(lang),
        get_contribute_language_option(lang),
        get_disable_option(lang),
        get_version_option(lang),
        get_default_option(lang),
    ]
}

pub fn get_tik_tok_options(lang: &str) -> Vec<CreateSelectMenuOption> {
    vec![
        get_download_option(lang),
        CreateSelectMenuOption::new(t!("tiktxk", locale = lang), tiktok::TIKTXK_URL),
        CreateSelectMenuOption::new(t!("tnktok", locale = lang), tiktok::TNKTOK_URL),
        CreateSelectMenuOption::new(t!("show_media_only_tiktok", locale = lang), "direct_tiktxk"),
        CreateSelectMenuOption::new(
            t!("show_original_tiktok", locale = lang),
            tiktok::TIKTOK_URL,
        ),
        get_remove_option(lang),
        get_set_language_option(lang),
        get_contribute_language_option(lang),
        get_disable_option(lang),
        get_version_option(lang),
        get_default_option(lang),
    ]
}
