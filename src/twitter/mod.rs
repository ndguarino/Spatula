use egg_mode::stream::{filter, StreamMessage};
use futures::{TryStreamExt};
use serenity::{
    model::id::ChannelId,
    prelude::*,
};
use std::sync::Arc;

pub async fn run_twitter_stream(ctx: Arc<Context>) {
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

    let handles = include!("config/twitter_handles.in");
    
    let stream = filter()
        .follow(&handles)
        .start(&token);

    let ctx_copy = Arc::clone(&ctx);

    stream.try_for_each(|m| async {
        if let StreamMessage::Tweet(tweet) = m {
            let channels = include!("config/discord_channels.in");

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

            println!("Received tweet from {}:\n{}\n", tweet.user.unwrap().name, tweet.text);
        }

        Ok(())
    }).await.unwrap()
}
