/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serde::Deserialize;
use serenity::model::{channel::Message, guild::Guild};
use serenity::prelude::*;
use std::fs;

use std::sync::RwLockReadGuard;

#[derive(Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub prefix: char,
}

/**
 * Returns a Vec<String> of each word in the message. Arguments start at index 1
 */
pub fn parse_args(msg: &Message) -> Vec<String> {
    // Get the content of the string
    let msg_content: String = msg.content.clone();

    println!("{}", msg_content);
    // Split the message by spaces
    let split_msg_iter = msg_content.split_whitespace();

    let mut split_msg: Vec<String> = vec![];
    for (i, s) in split_msg_iter.enumerate() {
        split_msg.push(String::from(s));
    }
    println!("{:?}", split_msg);
    return split_msg.clone();
}

pub fn parse_config() -> BotConfig {
    let contents = fs::read_to_string("config.toml").unwrap();
    let config: BotConfig = toml::from_str(&contents).unwrap();

    return config;
}

pub fn extract_guild_from_msg<'a>(ctx: &Context, msg: &Message) -> RwLockReadGuard<'a, RawRwLock> {
    let guild_arc = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).unwrap();
    let guild = guild_arc.read();

    guild

}