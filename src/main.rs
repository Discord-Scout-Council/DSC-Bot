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
        Reason,
        DispatchError::{NotEnoughArguments,TooManyArguments, CheckFailed, OnlyForGuilds, OnlyForOwners, LackingPermissions}
    },
    model::{
        channel::{Message, Reaction, ReactionType},
        gateway::{Activity, ActivityType, Ready},
        guild::Guild,
        id::{UserId, GuildId},
        user::{OnlineStatus, User},
    },
    prelude::*,
    utils::Colour,
};
use std::{collections::HashSet, env};

use pickledb::*;
use rusqlite::{params, Connection, Result};

use log::{debug, error, info};

mod checks;
mod commands;
mod util;
use crate::commands::{
    general::*, moderation::*, owner::*, settings::*,
};
use util::*;

mod prelude;

#[group]
#[commands(ping, about, serverinfo, botsuggest)]
struct General;


#[group]
#[commands(restart, initcache)]
struct Owner;

#[group]
#[commands(strike, strikelog, wordfilter, clearstrikes, modstrike, getcase)]
struct Moderation;


#[group]
#[commands(serversettings, resetsettings)]
struct Settings;

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("Logged in to Discord successfully");
        let activity = Activity::playing("with bans");
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
        debug!("Checking banned words list");
        let guild = &msg.guild_id.unwrap();
        if util::moderation::contains_banned_word(&msg.content, &guild.as_u64()) {
            msg.channel_id
                .send_message(&ctx.http, |m| {
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
        let mut db = util::data::get_pickle_database(
            msg.guild_id.unwrap().as_u64(),
            &String::from("points.db"),
        );
    }

    fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        let db = data::get_discord_banlist();
        if let error = db.execute("INSERT INTO dbans (userid,reason,guild_id,is_withdrawn) VALUES (?1,?2,?3,0)", params![banned_user.id.as_u64().to_string(),"No Reason Provided", guild_id.as_u64().to_string()]) {
            error!("Encountered an error adding a ban for {}", banned_user.name);
        };
        let blacklist_channel = ctx.http.get_channel(646545388576178178).unwrap();
        let blacklist_channel_id = blacklist_channel.id();
        let guild = ctx.http.get_guild(guild_id.as_u64().clone()).unwrap();

        if let error = blacklist_channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("New Ban Detected");
                e.fields(vec![
                    ("Server", &guild.name, false),
                    ("Name", &format!("{}#{}", &banned_user.name, &banned_user.discriminator), false),
                    ("ID", &banned_user.id.as_u64().to_string(), false)
                ]);
                if let Some(url) = &banned_user.avatar_url() {
                    e.thumbnail(url);
                }
                e
            });
            m
        }) {
            error!("Encountered an error trying to notify DSC about a new ban for {}", banned_user.name);
        }


    }

    fn guild_create(&self, ctx: Context, guild: Guild, _is_new: bool) {
        if _is_new {
            info!(
                "Joined new guild {}. Intializing guild settings.",
                &guild.name
            );
        }
        let mut cache = data::get_pickle_database(&guild.id.as_u64(), "settings.db");
        if let None = cache.get::<String>("qotd_channel") {
            data::init_guild_settings(&mut cache);
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
    kankyo::init().expect("Failed to load .env file");
    env_logger::init();

    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Error => {
            error!("Could not find discord token in environment");
            String::from("")
        }
    };

    let mut client = Client::new(token, Handler).expect("Err creating client");

    debug!("Getting owners");
    let owners = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
        Err(why) => panic!("Coudln't get application info: {:?}", why),
    };

    debug!("Initializing client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix(&env::var("DISCORD_PREFIX").unwrap())
                    .owners(owners)
                    .allow_dm(false)
            })
            .help(&HELP)
            .group(&GENERAL_GROUP)
            .group(&OWNER_GROUP)
            .group(&MODERATION_GROUP)
            .group(&SETTINGS_GROUP)
            .on_dispatch_error(|context, msg, error| {
                match error {
                    NotEnoughArguments { min, given } => {
                        let mut s = format!("Need {} arguments, only got {}.", min, given);
                        s.push_str(&" Try using `help <command>` to get usage.");

                        msg.channel_id.say(&context, &s);
                    },
                    TooManyArguments { max, given} => {
                        let mut s = format!("Too many arguments. Expected {}, got {}.", max, given);
                        s.push_str(" Try using `help <command>` to get usage.");

                        msg.channel_id.say(&context, &s);
                    },
                    CheckFailed ( stri, reason) => {
                        info!("{}", stri);
                        info!("{} failed to pass check {}", &msg.author.name, stri);

                        msg.channel_id.say(&context, "You do not have permission to use this command!");

                    }
                    _ => error!("Unhandled dispatch error.")
                }
            })
    );

    info!("Starting client");
    if let Err(err) = client.start() {
        error!("Client error: {:?}", err);
    }
}
