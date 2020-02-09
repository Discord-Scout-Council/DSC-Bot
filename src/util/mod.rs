use serenity::model::channel::Message;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub prefix: char
}

pub fn parse_args<'a>(msg: &'a Message) -> Vec<String> {

    // Get the content of the string
    let msg_content: String = msg.content.clone();

    // Split the message by spaces
    let split_msg_iter = msg_content.split_whitespace();

    let mut split_msg: Vec<String> = vec![];
    for (i, s) in split_msg_iter.enumerate() {
        split_msg[i] = String::from(s);
    }

    return split_msg.clone();
}

pub fn parse_config() -> BotConfig {
    let contents = fs::read_to_string("config.toml").unwrap();
    let config: BotConfig = toml::from_str(&contents).unwrap();

    return config;
}