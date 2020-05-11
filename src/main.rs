/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */
use serenity::{
    async_trait,
    http::Http,
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args, CommandGroup, CommandResult,
        DispatchError::{
            CheckFailed, CommandDisabled, NotEnoughArguments, OnlyForGuilds, TooManyArguments,
        },
        DispatchError,
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
use crate::commands::{general::*, moderation::*, owner::*, settings::*, verification::*, badges::*};
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


#[group]
#[commands(age, verify)]
struct Verification;

#[group]
#[commands(addbadge, delbadge)]
struct Badges;


struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        info!("Logged in to Discord successfully");
        let activity = Activity::playing("with vibes");
        ctx.set_presence(Some(activity), OnlineStatus::DoNotDisturb);
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if add_reaction.channel_id.as_u64() == &684577265425973285u64 {
            match verification::handle_verification_reaction(&ctx, add_reaction).await {
                Err(e) => error!("Error verifying user. {}", e),
                _ => (),
            }
        }
    }

    //* Points
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.is_private() {
            verification::handle_verification_file(&ctx, &msg).await;
        }
        //* Banned Words
        debug!("Checking banned words list");
        let guild = match &msg.guild_id {
            Some(id) => id,
            None => {
                debug!("Could not find guildid for private message");
                return;
            }
        };
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
                }).await
                .unwrap();
            let action = moderation::ModAction {
                target: msg.author.clone().id,
                moderator: ctx
                    .http
                    .get_current_application_info()
                    .await
                    .unwrap()
                    .id
                    .to_user(&ctx)
                    .await
                    .unwrap()
                    .clone(),
                reason: Some(String::from("Found a banned word")),
                details: None,
                action_type: moderation::ModActionType::BadWordDelete,
                guild: msg.guild_id.clone().unwrap(),
            };
            moderation::log_mod_action(action, &mut ctx.clone()).await;
            msg.delete(&ctx).await.unwrap();
        }
    }

    async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        let db = data::get_discord_banlist();
        let bans = guild_id.bans(&ctx).await.unwrap();

        let mut reason = &String::from("No reason provided");
        for (_i, b) in bans.iter().enumerate() {
            if b.user.id.as_u64() == banned_user.id.as_u64() {
                match &b.reason {
                    Some(r) => reason = r,
                    None => (),
                }
            }
        }
        if let Err(err) = db.execute(
            "INSERT INTO dbans (userid,reason,guild_id,is_withdrawn) VALUES (?1,?2,?3,0)",
            params![
                banned_user.id.as_u64().to_string(),
                &reason,
                guild_id.as_u64().to_string()
            ],
        ) {
            error!(
                "Encountered an error adding a ban for {}: {:?}",
                banned_user.name, err
            );
        };
        let blacklist_channel = ctx.http.get_channel(646545388576178178).await.unwrap();
        let blacklist_channel_id = blacklist_channel.id();
        let guild = ctx.http.get_guild(guild_id.as_u64().clone()).await.unwrap();

        if let Err(err) = blacklist_channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("New Ban Detected");
                e.fields(vec![
                    ("Server", &guild.name, false),
                    (
                        "Name",
                        &format!("{}#{}", &banned_user.name, &banned_user.discriminator),
                        false,
                    ),
                    ("ID", &banned_user.id.as_u64().to_string(), false),
                ]);
                if let Some(url) = &banned_user.avatar_url() {
                    e.thumbnail(url);
                }
                e
            });
            m
        }).await {
            error!(
                "Encountered an error trying to notify DSC about a new ban for {}: {:?}",
                banned_user.name, err
            );
        }
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: bool) {
        if _is_new {
            info!(
                "Joined new guild {}. Intializing guild settings.",
                &guild.name
            );
        }
        let mut cache = data::get_pickle_database(&guild.id.as_u64(), "settings.db");
        if let None = cache.get::<String>("modlogs_channel") {
            data::init_guild_settings(&mut cache);
        }
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
        let db = data::get_discord_banlist();
        let user_id = new_member.user.id;
        let member_id = user_id.as_u64();
        let mut is_banned = false;
        let mut reason = String::from("No reason provided");
        {
        let mut stmt = db
            .prepare("SELECT reason FROM dbans WHERE userid = (?)")
            .unwrap();
        let mut ban_result = stmt.query(params![&member_id.to_string()]).unwrap();
        if let Ok(o) = ban_result.next() {
            if let Some(r) = o {
                is_banned = true;
                reason = r.get(0).unwrap();
            }
        }
    }
        let guild_arc = guild_id.to_guild_cached(&ctx).await.unwrap();
        let guild = guild_arc.read().await;

        let settings = data::get_pickle_database(&guild_id.as_u64(), "settings.db");
        let alert_channel: ChannelId;
        let temp_channel = match settings.get::<u64>("modlogs_channel") {
            Some(channel) => channel,
            None => 0u64,
        };
        if temp_channel == 0 {
            alert_channel = match guild.system_channel_id {
                Some(c) => c,
                None => {
                    error!("Error fetching mod logs channel: Channel not set and System Channel does not exist.");
                    return;
                }
            }
        } else {
            alert_channel = temp_channel.into();
        }
        if is_banned {
            let user = &ctx.http.get_user(*new_member.user.id.as_u64()).await.unwrap();
            match alert_channel.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Alert!");
                    e.description("A banned user has joined the server.");
                    e.field(
                        "User",
                        format!("{}#{}", user.name, user.discriminator),
                        true,
                    );
                    e.field("Reason", reason, true);
                    e.footer(|f| {
                        f.text(format!("DSC Bot | Powered by Rusty Developers"));
                        f
                    });
                    e.colour(Colour::RED);

                    e
                });
                m
            }).await {
                Err(err) => {
                    error!(
                        "Encountered an error warning {} about {}#{}: {:?}",
                        &guild.name, user.name, user.discriminator, err
                    );
                }
                _ => (),
            }
        }
    }
}

#[hook]
async fn on_dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {

    match error {
        NotEnoughArguments { min, given } => {
            let mut s = format!("Need {} arguments, only got {}.", min, given);
            s.push_str(&" Try using `help <command>` to get usage.");

            match msg.channel_id.say(&ctx, &s).await {
                Err(err) => error!("Error responding to invalid arguments: {:?}", err),
                Ok(_msg) => (),
            }
        }
        TooManyArguments { max, given } => {
            let mut s = format!("Too many arguments. Expected {}, got {}.", max, given);
            s.push_str(" Try using `help <command>` to get usage.");

            match msg.channel_id.say(&ctx, &s).await {
                Err(err) => error!("Error responding to invalid arguments: {:?}", err),
                Ok(_msg) => (),
            }
        }
        CheckFailed(stri, _reason) => {
            info!("{}", stri);
            info!("{} failed to pass check {}", &msg.author.name, stri);

            match msg
                .channel_id
                .say(&ctx, "You do not have permission to use this command!").await
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
                .say(&ctx, "Please run this command in a Server!").await
            {
                Err(err) => {
                    error!("Error sending invalid context msg to {}", &msg.author.name)
                }
                _ => (),
            }
        },
        CommandDisabled(stri) => {
            if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
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
            }).await {
                error!("Error sending disabled command message to {}", &msg.author.name);
            }
        }
        _ => error!("Unhandled dispatch error."),
    }
}

#[hook]
async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
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
    }).await {
        error!("Error sending error message {:?}", err);
    }
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
    .after(after)
    .on_dispatch_error(on_dispatch_error)
    .group(&GENERAL_GROUP)
    .group(&MODERATION_GROUP)
    .group(&OWNER_GROUP)
    .group(&SETTINGS_GROUP)
    .group(&VERIFICATION_GROUP)
    .group(&BADGES_GROUP)
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
