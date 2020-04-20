use serenity::{
    framework::standard::{macros::group, StandardFramework},
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::collections::HashSet;
mod commands;
mod util;
use crate::commands::general::*;

#[group]
#[commands(ping,about)]
struct General;

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} logged in successfully!", ready.user.name);
    }
}

fn main() {
    let config: util::BotConfig = util::parse_config();

    let token = config.token.clone();

    let mut client = Client::new(token, Handler).expect("Err creating client");

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix(&config.prefix.to_string()))
            .group(&GENERAL_GROUP),
    );

    if let Err(err) = client.start() {
        eprintln!("{:?}", err);
    }
}

