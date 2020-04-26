/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use rand::Rng;
use serenity::{
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::{Activity, ActivityType, Ready},
        id::UserId,
        user::OnlineStatus,
        guild::Guild,
    },
    prelude::*,
    utils::Colour,
};
use std::collections::HashSet;

use pickledb::*;
use rusqlite::{params, Connection, Result};

mod checks;
mod commands;
mod util;
use crate::commands::{general::*, moderation::*, owner::*, points::*, qotd::*, settings::*};
use util::*;

#[group]
#[commands(ping, about, serverinfo)]
struct General;

#[group]
#[commands(points, leaderboard)]
struct Points;

#[group]
#[commands(restart, initcache)]
struct Owner;

#[group]
#[commands(strike, strikelog, wordfilter, clearstrikes)]
struct Moderation;

#[group]
#[commands(qotd)]
struct Qotd;

#[group]
#[commands(serversettings, resetsettings)]
struct Settings;

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} logged in successfully!", ready.user.name);
        let activity = Activity::streaming("Pioneering", "https://devosmium.xyz");
        ctx.set_presence(Some(activity), OnlineStatus::DoNotDisturb);
    }
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let msg = reaction.message(ctx.http).unwrap();
        let reactions = msg.reactions;
        for r in &reactions {
            match &r.reaction_type {
                ReactionType::Custom { animated, id, name } => {
                    if id.as_u64() == &701900676313383092 {
                        if (r.count >= 2) {
                            println!("Starboarded!");
                        }
                    }
                }
                ReactionType::Unicode(emoji) => {
                    if emoji == "⭐" {
                        if (r.count >= 1) {
                            println!("Starboarded!");
                        }
                    }
                }
                __ => (),
            }
        }
    }

    //* Points
    fn message(&self, ctx: Context, msg: Message) {
        //* Banned Words
        let guild = &msg.guild_id.unwrap();
        if util::moderation::contains_banned_word(&msg.content, &guild.as_u64()) {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    let mut mention = String::from("<@");
                    mention.push_str(&msg.author.id.as_u64().to_string());
                    mention.push_str(">");

                        e.title("Warning - Bad Language");
                        e.description("Do not use poor language or slurs in this server.");
                        e.fields(vec![("User:", mention, false)]);

                        e.color(Colour::RED);

                        e
                    });

                    m
                })
                .unwrap();
            let action = moderation::ModAction {
                target: msg.author.clone().id,
                moderator: ctx
                    .http
                    .get_current_application_info()
                    .unwrap()
                    .id
                    .to_user(&ctx)
                    .unwrap()
                    .clone(),
                reason: Some(String::from("Found a banned word")),
                details: None,
                action_type: moderation::ModActionType::BadWordDelete,
                guild: msg.guild_id.clone().unwrap(),
            };
            moderation::log_mod_action(action, &mut ctx.clone());
            msg.delete(&ctx).unwrap();
        }
        let config = util::parse_config();
        let mut db = util::data::get_pickle_database(
            msg.guild_id.unwrap().as_u64(),
            &String::from("points.db"),
        );
        /*if let None = db.get::<u64>(&msg.author.id.to_string()) {
            println!("Did not find user {}", msg.author.id);
            db.set(&msg.author.id.to_string(), &0).unwrap();
        }*/

        if !msg.content.starts_with(config.prefix) {
            if !msg
                .channel_id
                .name(ctx)
                .unwrap()
                .contains(&String::from("bot"))
            {
                let points: u64 = rand::thread_rng().gen_range(1, 4);
                let current_points: u64 = match db.get(&msg.author.id.to_string()) {
                    Some(i) => i,
                    None => 0,
                };
                println!("Current points: {}", current_points);
                let total_points = current_points + points;
                println!("Total Points: {}", total_points);

                db.set(&msg.author.id.to_string(), &total_points)
                    .expect("Could not add points");
            }
        }
    }

    fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        let mut cache = data::get_pickle_database(&guild.id.as_u64(), "cache.db");
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
    let strikes_conn = Connection::open("strikes.db").unwrap();
    strikes_conn
        .execute(
            "CREATE TABLE IF NOT EXISTS strikes (
                                    id INTEGER PRIMARY KEY,
                                    userid TEXT NOT NULL,
                                    reason TEXT)",
            params![],
        )
        .unwrap();

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
            .group(&GENERAL_GROUP)
            .group(&POINTS_GROUP)
            .group(&OWNER_GROUP)
            .group(&MODERATION_GROUP)
            .group(&QOTD_GROUP)
            .group(&SETTINGS_GROUP),
    );

    if let Err(err) = client.start() {
        eprintln!("{:?}", err);
    }
}
