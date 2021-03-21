use serenity::framework::standard::macros::group;
mod emoji_commands;
use crate::emoji::emoji_commands::NEW_EMOJI_COMMAND;

#[group]
#[commands(new_emoji)]
pub struct Emoji;
