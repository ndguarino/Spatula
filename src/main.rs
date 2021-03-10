use egg_mode::stream::{filter, StreamMessage};
use futures::{TryStreamExt};
use serenity::{
    async_trait,
    model::{id::{GuildId, ChannelId}, gateway::{Ready}},
    prelude::*,
};
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::Arc;

async fn run_twitter_stream(ctx: Arc<Context>) {
    let consumer_token =  std::env::var("TWITTER_CONSUMER_TOKEN").unwrap();
    let consumer_secret = std::env::var("TWITTER_CONSUMER_SECRET").unwrap();
    let access_token =  std::env::var("TWITTER_ACCESS_TOKEN").unwrap();
    let access_secret = std::env::var("TWITTER_ACCESS_SECRET").unwrap();
    let con_token = egg_mode::KeyPair::new(consumer_token, consumer_secret);
    let access_token = egg_mode::KeyPair::new(access_token, access_secret);
    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: access_token,
    };
    
    let stream = filter()
        .follow(&[69713509])
        //.filter_level(FilterLevel::Medium)
        .start(&token);

    let ctx_copy = Arc::clone(&ctx);


    stream.try_for_each(|m| async {
        if let StreamMessage::Tweet(tweet) = m {
            let channels = vec![766449729562738728u64, 440601976804343808, 480137199350710305];
            //let users = vec![];

            for channel in channels.iter() {
                if let Err(why) = ChannelId(*channel).send_message(&ctx_copy, |m| m.embed(|embed| {
                    let newtweet = tweet.clone();
                    embed.title("New Tweet from @lusternia");
                    embed.description(newtweet.text);
                    if let (Some(url), name) = (&newtweet.user.as_ref().unwrap().url, &newtweet.user.as_ref().unwrap().name) {
                        embed.url(url);
                        embed.author(|a| {
                            a.name(name);
                            a.url(url);

                            a
                        });
                    };

                    embed
                })).await {
                    eprintln!("Twitter stream error: {:?}", why);
                }
            }

            /*
            for user in users.iter() {
                if let Err(why) = ChannelId(channel).send_message(&ctx, |m| m.embed(|_| {
                    embed
                })).await {
                    eprintln!("Twitter stream error: {:?}", why);
                }
            }
            */

            println!("Received tweet from {}:\n{}\n", tweet.user.unwrap().name, tweet.text);
        }
        Ok(())
    }).await.expect("Stream Error")
}

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
                    run_twitter_stream(Arc::clone(&c_ctx)).await;
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
