use serenity::client::bridge::gateway::GatewayIntents;
use serenity::{
    async_trait,
    framework::{
        standard::{macros::hook, DispatchError, Reason},
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::GuildId},
    prelude::*,
};
use songbird::SerenityInit;
use tracing::{info, warn};
use tracing_log::env_logger;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
mod data;
mod emoji;
mod music;
mod tasks;

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::CheckFailed(_, reason) => match reason {
            Reason::Log(log) => info!("{:?}", log),
            Reason::User(reason) => {
                let _ = msg.reply_ping(ctx, reason).await;
            }
            _ => {}
        },
        DispatchError::Ratelimited(duration) => {
            if duration.as_secs() == 0 {
                let _ = msg
                    .channel_id
                    .say(
                        ctx,
                        format!("Try this again in {}ms.", duration.as_millis()),
                    )
                    .await;
            } else {
                let _ = msg
                    .channel_id
                    .say(
                        ctx,
                        format!("Try this again in {} seconds.", duration.as_secs()),
                    )
                    .await;
            }
        }
        e => warn!("{:?}", e),
    }
}

struct DiscordHandler {}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache ready.");
        println!("Cache ready.");
        tasks::start(&ctx.clone()).await;
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("!");
            c.disabled_commands(
                vec!["play", "p", "queue", "skip"]
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect(),
            )
        })
        .group(&emoji::EMOJI_GROUP)
        .group(&music::MUSIC_GROUP)
        .on_dispatch_error(dispatch_error)
        .after(|ctx, msg, cmd_name, error| {
            Box::pin(async move {
                if let Err(e) = error {
                    warn!("Error with command {}, {:?}", cmd_name, e);
                    let _ = msg
                        .channel_id
                        .say(
                            ctx,
                            format!(
                                "Command returned an error, {:?}, please report this to Ianir",
                                e
                            ),
                        )
                        .await;
                }
            })
        });

    let token = std::env::var("DISCORD_TOKEN").expect("Could not load DISCORD_TOKEN");

    let mut client = Client::builder(&token)
        .event_handler(DiscordHandler {})
        .framework(framework)
        .register_songbird()
        .intents(
            GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_VOICE_STATES
                | GatewayIntents::GUILD_EMOJIS
                | GatewayIntents::GUILD_MEMBERS
                | GatewayIntents::GUILDS,
        )
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        //data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<data::ReqwestClientContainer>(Default::default());
        data.insert::<music::queue::QueueMap>(Default::default());
        //data.insert::<PrefixCache>(Default::default());
    }

    client.start_autosharded().await.expect("Client error");

    Ok(())
}
