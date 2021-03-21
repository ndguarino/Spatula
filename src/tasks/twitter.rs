use egg_mode::error::Error;
use egg_mode::{
    stream::{filter, StreamMessage},
    KeyPair, Token,
};
use futures::TryStreamExt;
use serenity::{model::id::ChannelId, prelude::*};

pub async fn start(ctx: Context) {
    let twitter_consumer_token =
        std::env::var("TWITTER_CONSUMER_TOKEN").expect("Missing Twitter token");
    let twitter_consumer_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("Missing Twitter token");
    let twitter_access_token =
        std::env::var("TWITTER_ACCESS_TOKEN").expect("Missing Twitter token");
    let twitter_access_secret =
        std::env::var("TWITTER_ACCESS_SECRET").expect("Missing Twitter token");
    let con_token = KeyPair::new(twitter_consumer_token, twitter_consumer_secret);
    let access_token = KeyPair::new(twitter_access_token, twitter_access_secret);
    let token = Token::Access {
        consumer: con_token,
        access: access_token,
    };

    loop {
        let _ = async {
            run_twitter_stream(&ctx.clone(), &token)
                .await
                .expect("Twitter Stream Error")
        };
    }
}

pub async fn run_twitter_stream(ctx: &Context, token: &Token) -> Result<(), Error> {
    let handles = include!("config/twitter_handles.in");

    let stream = filter().follow(&handles).start(&token);

    let ctx_copy = &ctx.clone();

    stream
        .try_for_each(|m| async {
            if let StreamMessage::Tweet(tweet) = m {
                let channels = include!("config/discord_channels.in");

                for channel in channels.iter() {
                    if let Err(why) = ChannelId(*channel)
                        .send_message(&ctx_copy, |m| {
                            m.embed(|embed| {
                                let newtweet = tweet.clone();
                                embed.title("New Tweet from @lusternia");
                                embed.description(newtweet.text);
                                if let (Some(url), name) = (
                                    &newtweet.user.as_ref().unwrap().url,
                                    &newtweet.user.as_ref().unwrap().name,
                                ) {
                                    embed.url(url);
                                    embed.author(|a| {
                                        a.name(name);
                                        a.url(url);

                                        a
                                    });
                                };

                                embed
                            })
                        })
                        .await
                    {
                        eprintln!("Twitter stream error: {:?}", why);
                    }
                }

                println!(
                    "Received tweet from {}:\n{}\n",
                    tweet.user.unwrap().name,
                    tweet.text
                );
            }

            Ok(())
        })
        .await
}
