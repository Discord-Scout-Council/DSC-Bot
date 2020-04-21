use serde::Deserialize;
use serenity::model::{
    channel::{Message, Reaction, GuildChannel},
    id::{ChannelId, GuildId},
};
use serenity::prelude::*;
use std::fs;

use std::collections::HashMap;

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

pub fn find_guild_channel_by_id(ctx: &Context, id: u64, guild: GuildId) -> Option<ChannelId> {
    let channels = guild.channels(&ctx).unwrap();
    println!("{:?}", channels);
    let mut target_channel: Option<ChannelId> = None;
    for (key, value) in &channels {
        if key.as_u64() == &id {
            target_channel = Some(*key);
        } else {
            target_channel = None;
        }
    }

    target_channel
}

pub fn starboard(ctx: &Context, reaction: &Reaction) {
    let star_message_id = reaction.message_id;
    let star_message = reaction.channel_id.message(&ctx.http, star_message_id).unwrap();
    let star_channel = find_guild_channel_by_id(
        &ctx,
        697962591149883434,
        reaction.guild_id.unwrap(),
    ).unwrap();
    star_channel.send_message(&ctx.http, |m| {
        m.embed(|mut e| {
            e.title(&star_message.author.name);
            e.description(star_message.content);

            let avatar_url = match star_message.author.avatar_url() {
                Some(url) => url,
                None => star_message.author.default_avatar_url()
            };

            e.thumbnail(avatar_url);

            e
        });

        m
    });
}