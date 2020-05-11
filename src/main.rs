/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */
use serenity::{
    async_trait,
    http::Http,
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult,
        DispatchError::{
            CheckFailed, CommandDisabled, NotEnoughArguments, OnlyForGuilds, TooManyArguments,
        },
        HelpOptions, StandardFramework,
    },
    model::{
        channel::{Message, Reaction},
        gateway::{Activity, Ready},
        guild::{Guild, Member},
        id::{ChannelId, GuildId, UserId},
        user::{OnlineStatus, User},
    },
    prelude::*,
    utils::Colour,
};
use std::{collections::HashSet, env};

use rusqlite::params;

use log::{debug, error, info};

mod checks;
mod commands;
mod util;
/*use crate::commands::{
    badges::*, general::*, moderation::*, owner::*, settings::*, verification::*,
};*/
use crate::commands::{general::*, moderation::*, owner::*, settings::*};
use util::*;

mod prelude;

#[group]
#[commands(ping, about, serverinfo, botsuggest, privacy)]
struct General;

#[group]
#[commands(restart, initcache)]
struct Owner;


#[group]
#[commands(
    strike,
    strikelog,
    wordfilter,
    clearstrikes,
    modstrike,
    getstrike,
    runuser,
    syncbans,
    advise,
    modban,
    bans,
    raid,
    unraid,
)]
struct Moderation;

#[group]
#[commands(serversettings, resetsettings)]
struct Settings;

/*
#[group]
#[commands(age, verify)]
struct Verification;

#[group]
#[commands(addbadge)]
struct Badges;
*/

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }
}

#[help]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}

#[tokio::main]
async fn main() {
    kankyo::init().expect("Failed to load .env file");
    env_logger::init();

    let token = match env::var("DISCORD_TOKEN") {
        Ok(t) => t,
        Err(_err) => {
            error!("Could not find discord token in environment");
            String::from("")
        }
    };

    let http = Http::new_with_token(&token);


    debug!("Getting owners");
    let owners = match http.get_current_application_info().await {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        }
        Err(why) => panic!("Coudln't get application info: {:?}", why),
    };

    debug!("Initializing Framework");
    let framework = StandardFramework::new()
    .configure(|c| c
        .owners(owners)
        .prefix(&env::var("DISCORD_PREFIX").unwrap())
    )
    .group(&GENERAL_GROUP)
    .group(&MODERATION_GROUP)
    .group(&OWNER_GROUP)
    .group(&SETTINGS_GROUP)
    .help(&HELP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    debug!("Initializing client");
    let mut disabled_commands: HashSet<String> = HashSet::new();
    disabled_commands.insert("servers".to_string());
/*    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix(&env::var("DISCORD_PREFIX").unwrap())
                    .owners(owners).disabled_commands(disabled_commands)
            })
            .help(&HELP)
            .group(&GENERAL_GROUP)
            .group(&OWNER_GROUP)
            .group(&MODERATION_GROUP)
            .group(&SETTINGS_GROUP)
            .group(&VERIFICATION_GROUP)
            .group(&BADGES_GROUP)
            .on_dispatch_error(|context, msg, error| match error {
                NotEnoughArguments { min, given } => {
                    let mut s = format!("Need {} arguments, only got {}.", min, given);
                    s.push_str(&" Try using `help <command>` to get usage.");

                    match msg.channel_id.say(&context, &s) {
                        Err(err) => error!("Error responding to invalid arguments: {:?}", err),
                        Ok(_msg) => (),
                    }
                }
                TooManyArguments { max, given } => {
                    let mut s = format!("Too many arguments. Expected {}, got {}.", max, given);
                    s.push_str(" Try using `help <command>` to get usage.");

                    match msg.channel_id.say(&context, &s) {
                        Err(err) => error!("Error responding to invalid arguments: {:?}", err),
                        Ok(_msg) => (),
                    }
                }
                CheckFailed(stri, _reason) => {
                    info!("{}", stri);
                    info!("{} failed to pass check {}", &msg.author.name, stri);

                    match msg
                        .channel_id
                        .say(&context, "You do not have permission to use this command!")
                    {
                        Err(err) => error!("Error responding to failed check: {:?}", err),
                        Ok(_msg) => (),
                    }
                }
                OnlyForGuilds => {
                    info!(
                        "{} tried to use a guild-only command in DMs",
                        &msg.author.name
                    );
                    match msg
                        .channel_id
                        .say(&context, "Please run this command in a Server!")
                    {
                        Err(err) => {
                            error!("Error sending invalid context msg to {}", &msg.author.name)
                        }
                        _ => (),
                    }
                },
                CommandDisabled(stri) => {
                    if let Err(err) = msg.channel_id.send_message(&context, |m| {
                        m.embed(|e| {
                            e.title("Command Error");
                            e.description("That command has been disabled.");
                            e.colour(Colour::RED);
                            e.footer(|f| {
                                f.text("DSC Bot | Powered by Rusty Development");
                                f
                            });
                            e
                        });
                        m
                    }) {
                        error!("Error sending disabled command message to {}", &msg.author.name);
                    }
                }
                _ => error!("Unhandled dispatch error."),
            })
            .after(|ctx, msg, cmd_name, error | {
                if let Err(err) = error {
                    error!("Error in {}: {:?}", cmd_name, err);
                    if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.title("Command Error");
                            e.description("There was an error running the command. Please report to DSC Tech Team");
                            e.footer(|f| {
                                f.text("DSC Bot | Powered by Rusty Development");
                                f
                            });
                            e.colour(Colour::RED);
                            e
                        });
                        m
                    }) {
                        error!("Error sending error message {:?}", err);
                    }
                }
            }),
    );
*/
    info!("Starting client");
    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    }
}
