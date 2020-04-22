/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::{
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::Ready,
        id::UserId,
    },
    prelude::*,
};
use std::collections::HashSet;
mod checks;
mod commands;
mod util;
use crate::commands::general::*;
use util::{find_guild_channel_by_id, starboard};

#[group]
#[commands(ping, about)]
struct General;

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} logged in successfully!", ready.user.name);
    }
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let msg = reaction.message(&ctx.http).unwrap();
        let reactions = msg.reactions;
        for r in &reactions {
            match &r.reaction_type {
                ReactionType::Custom { animated, id, name } => {
                    if id.as_u64() == &701900676313383092 {
                        if (r.count >= 2) {
                            starboard(&ctx, &reaction);
                        }
                    }
                }
                ReactionType::Unicode(emoji) => {
                    if emoji == "⭐" {
                        if (r.count >= 1) {
                            starboard(&ctx, &reaction);
                        }
                    }
                }
                __ => (),
            }
        }
    }
}

#[help]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
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
        }
        Err(why) => panic!("Coudln't get application info: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix(&config.prefix.to_string()).owners(owners))
            .help(&HELP)
            .group(&GENERAL_GROUP),
    );

    if let Err(err) = client.start() {
        eprintln!("{:?}", err);
    }
}
