/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::{
    framework::standard::{
        macros::{help,group}, StandardFramework, Args, CommandResult, HelpOptions, help_commands, CommandGroup},
    model::{channel::{Reaction, ReactionType, Message}, gateway::Ready, id::UserId},
    prelude::*,
};
use std::{collections::HashSet};
use rand::Rng;

use pickledb::*;

mod commands;
mod util;
mod checks;
use crate::commands::{general::*, points::*};

#[group]
#[commands(ping,about)]
struct General;

#[group]
#[commands(points)]
struct Points;

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

    fn message(&self, ctx: Context, msg: Message) {
        let mut db = PickleDb::load_yaml("points.db", PickleDbDumpPolicy::AutoDump).unwrap();
        /*if let None = db.get::<u64>(&msg.author.id.to_string()) {
            println!("Did not find user {}", msg.author.id);
            db.set(&msg.author.id.to_string(), &0).unwrap();
        }*/

        if !msg.channel_id.name(ctx).unwrap().contains(&String::from("bot")) {
            let points: u64 = rand::thread_rng().gen_range(5,11);
            let current_points: u64 = match db.get(&msg.author.id.to_string()) {
                Some(i) => i,
                None => 0,
            };
            println!("Current points: {}", current_points);
            let total_points = current_points + points;
            println!("Total Points: {}", total_points);

            db.set(&msg.author.id.to_string(),  &total_points).expect("Could not add points");

        }
    }
}

#[help]
fn help(context: &mut Context, msg: &Message, args: Args, help_options: &'static HelpOptions, groups: &[&'static CommandGroup], owners: HashSet<UserId>) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

fn main() {

    // Load points database
    if let Error = PickleDb::load_yaml("points.db", PickleDbDumpPolicy::AutoDump) {
        PickleDb::new_yaml("points.db", PickleDbDumpPolicy::AutoDump);
    }
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
            .help(&HELP)
            .group(&GENERAL_GROUP)
            .group(&POINTS_GROUP),
    );

    if let Err(err) = client.start() {
        eprintln!("{:?}", err);
    }
}

