use std::sync::Once;
use serenity::prelude::*;
use async_std::task;
mod twitter;

static START: Once = Once::new();

pub async fn start(ctx: &Context) {
    START.call_once(|| {
        task::spawn(twitter::start(ctx.clone()));
    });
}