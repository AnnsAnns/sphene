
use thorium::db::Server;

use crate::Context;

#[derive(Debug, poise::ChoiceParameter)]
pub enum Choices {
    Twitter,
    Bluesky,
    Instagram,
    Tiktok,
}

pub const REGEX_URL_EXTRACTOR: &str = r"\b(?:https?:\/\/|<)[^\s>]+(?:>|)\b";

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
pub trait ToLanguageString {
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

pub fn parse_choice(choice: &Choices, mut server: Server, change_to: bool) -> Server {
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

// Gets relevent id, if guild id is present, it returns the guild id, else it returns the author id
pub fn get_relevant_id(ctx: Context<'_>) -> u64 {
    if ctx.guild_id().is_some() {
        ctx.guild_id().unwrap().get()
    } else {
        ctx.author().id.get()
    }
}
