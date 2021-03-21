use serenity::{client::bridge::gateway::ShardManager, prelude::*};
use std::sync::Arc;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ReqwestClientContainer;
impl TypeMapKey for ReqwestClientContainer {
    type Value = reqwest::Client;
}
