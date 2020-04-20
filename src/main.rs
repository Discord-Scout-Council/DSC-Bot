use serenity::{
    framework::standard::{macros::group, StandardFramework},
    model::{channel::{Reaction, ReactionType}, gateway::Ready},
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
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let msg = reaction.message(ctx.http).unwrap(); 
        let reactions = msg.reactions;
        for r in &reactions {
            match &r.reaction_type {
                ReactionType::Custom{animated,id,name} => if id.as_u64() == &701900676313383092 {
                    if (r.count >= 2) {
                        println!("Starboarded!");
                    }
                },
                ReactionType::Unicode(emoji) => if emoji == "⭐" {
                    if (r.count >= 1) {
                        println!("Starboarded!");
                    }
                },
                __ => (),
            }
        }
    }
}

fn main() {
    let config: util::BotConfig = util::parse_config();

    let token = config.token.clone();

    let mut client = Client::new(token, Handler).expect("Err creating client");

    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Coudln't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c
                .prefix(&config.prefix.to_string())
                .owners(owners))
            .group(&GENERAL_GROUP),
    );

    if let Err(err) = client.start() {
        eprintln!("{:?}", err);
    }
}

