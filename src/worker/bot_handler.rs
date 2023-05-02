use std::collections::HashMap;
use reqwest::Url;
use songbird::ConnectionInfo;
use songbird::id::{ChannelId, GuildId, UserId};
use tokio_tungstenite::tungstenite::connect;
use crate::config::Config;

pub async fn initialize_bot_instance(voice_channel_id: u64, guild_id: u64, config: &Config)  {
    let connection_info = ConnectionInfo {
        channel_id: Some(ChannelId(voice_channel_id)),
        guild_id: GuildId(guild_id),
        user_id: UserId(config.config.discord_bot_id),
        token: "".to_string(),
        session_id: "".to_string(),
        endpoint: "".to_string(),
    };
    // TODO: Build into Struct instead of Hashmap
    //TODO: Maybe cache this
    //TODO: Proper error handling to auto retry a few times before failing
    let resp = reqwest::get("https://discord.com/api/gateway")
        .await
        .unwrap()
        .json::<HashMap<String, String>>()
        .await
        .unwrap();
    let (mut socket, response) = connect(
        Url::parse(resp.get("url").expect("No URL")).unwrap()
    ).expect("Can't connect");
    loop {
        let msg = socket.read_message().expect("Error reading message");
        println!("Received: {}", msg);
    }
}