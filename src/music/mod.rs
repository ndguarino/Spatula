use serenity::framework::standard::macros::group;
pub mod commands;
pub mod events;
pub mod playlists;
pub mod queue;
use commands::{play::*, queue::*, skip::*};

#[group]
#[commands(play, skip, queue)]
struct Music;
