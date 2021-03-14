use serenity::{
    async_trait,
    model::{id::GuildId, gateway::{Ready}},
    prelude::*,
};
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::Arc;
mod twitter;

struct DiscordHandler {
    twitter_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let ctx = Arc::new(ctx);

        if !self.twitter_loop_running.load(Ordering::Relaxed) {
            let c_ctx = Arc::clone(&ctx);

            async_std::task::spawn(async move {
                loop {
                    crate::twitter::run_twitter_stream(Arc::clone(&c_ctx)).await;
                }
            });

            self.twitter_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

#[async_std::main]
async fn main() {
    let framework = serenity::framework::standard::StandardFramework::new()
        .configure(|c| c.prefix("!"));

    let token = std::env::var("DISCORD_TOKEN").unwrap();

    let mut client = Client::builder(&token)
        .event_handler(DiscordHandler {
            twitter_loop_running: AtomicBool::new(false),
        })
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}
