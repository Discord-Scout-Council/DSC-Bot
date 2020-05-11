/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use log::{debug, error, info, warn};
use rusqlite::params;
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::id::UserId;
use serenity::utils::Colour;
use serenity::{model::channel::{Message, ChannelType}, model::user::User, prelude::*};

use std::collections::HashMap;

use crate::checks::*;

use std::cmp::Ordering;

use crate::util::{
    data::{
        get_badge_db, get_discord_banlist, get_global_pickle_database, get_pickle_database,
        get_strike_database,
    },
    moderation::*,
};

struct Strike {
    user: UserId,
    reason: Option<String>,
    moderator: UserId,
}

#[derive(Eq, Hash)]
struct StrikeLog {
    user: UserId,
    reason: String,
    moderator: UserId,
    case_id: String,
}

struct DscBan {
    userid: String,
    reason: String,
}

impl Ord for StrikeLog {
    fn cmp(&self, other: &Self) -> Ordering {
        self.case_id
            .parse::<u32>()
            .unwrap()
            .cmp(&other.case_id.parse::<u32>().unwrap())
    }
}
impl PartialOrd for StrikeLog {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for StrikeLog {
    fn eq(&self, other: &Self) -> bool {
        self.case_id.parse::<u32>().unwrap() == other.case_id.parse::<u32>().unwrap()
    }
}

#[command]
#[description = "Adds a strike to the mentioned user"]
#[only_in(guilds)]
#[usage("<@User> <Reason>")]
#[min_args(2)]
#[checks(Moderator)]
async fn strike(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let strike_conn = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let strike = Strike {
        user: args.parse::<UserId>().unwrap(),
        reason: Some(String::from(args.advance().rest())),
        moderator: msg.author.clone().into(),
    };
    strike_conn
        .execute(
            "INSERT INTO strikes (userid, reason, moderator,is_withdrawn) VALUES (?1, ?2, ?3, 0)",
            params![
                strike.user.as_u64().to_string(),
                strike.reason,
                strike.moderator.as_u64().to_string()
            ],
        )
        .unwrap();

    msg.channel_id.say(&ctx.http, "Struck the user.").await.unwrap();
    let action = ModAction {
        target: strike.user,
        moderator: msg.author.clone(),
        action_type: ModActionType::Strike,
        reason: strike.reason,
        details: None,
        guild: msg.guild_id.unwrap(),
    };
    log_mod_action(action, ctx);

    Ok(())
}

#[command]
#[description = "Displays a list of strikes given to a user"]
#[only_in(guilds)]
#[min_args(1)]
#[checks(Moderator)]
async fn strikelog(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let strike_conn = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let target_user = args.parse::<UserId>().unwrap();
    let mut strikes: Vec<StrikeLog> = Vec::new();
{
    let mut stmt = strike_conn
        .prepare("SELECT reason,moderator,id FROM strikes WHERE userid = (?)")
        .unwrap();

    let mut rows = stmt
        .query(params![target_user.as_u64().to_string()])
        .unwrap();

    
    while let Some(row) = rows.next().unwrap() {
        let reason = row.get::<usize, String>(0).unwrap().clone();
        let moderator = row.get::<usize, String>(1).unwrap().clone();
        let case_id = row.get::<usize, u32>(2).unwrap().clone();
        let strike = StrikeLog {
            user: target_user,
            moderator: moderator.parse::<u64>().unwrap().into(),
            reason: reason,
            case_id: case_id.to_string(),
        };
        strikes.push(strike);
    }
}
    let mut result_vec: Vec<(String, String, bool)> = Vec::new();

    for (_i, r) in strikes.iter().enumerate() {
        result_vec.push((format!("Case #{}", r.case_id), r.reason.clone(), false));
    }

    let target_user_name = &target_user.to_user(&ctx.http).await.unwrap().name;
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            let mut title = String::from("Strikes for ");
            title.push_str(&target_user_name);
            e.title(title);

            e.fields(result_vec);

            let mut footer = String::from("Requested by ");
            footer.push_str(&msg.author.name);
            e.footer(|f| {
                f.text(footer);
                f
            });

            e
        });

        m
    }).await?;

    return Ok(());
}

#[command]
#[description = "Manages the bad words filter"]
#[sub_commands(add)]
async fn wordfilter(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Adds a word to the bad words list"]
#[checks(Moderator)]
#[sub_commands(global)]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(&ctx.http, "Called word management").await?;
    let guild = &msg.guild_id.unwrap();
    let mut db = get_pickle_database(guild.as_u64(), "banned_words.db");
    match db.get::<i32>(&args.rest()) {
        Some(_i) => {
            msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Word Filter");
                    e.description("That word is already filtered!");
                    e.colour(Colour::RED);

                    e
                });

                m
            }).await?;
        }
        None => {
            if let Err(err) = db.set(&args.rest(), &1) {
                error!("Failed to add local banned word: {:?}", err);
            };
            msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Word Filter");

                    let mut description = String::from("Added ");
                    description.push_str(&args.rest());
                    description.push_str(" to the server word filter");
                    e.description(description);
                    e.colour(Colour::DARK_GREEN);

                    e
                });

                m
            }).await?;
        }
    }

    Ok(())
}

#[command]
#[description = "Adds a word to the global list"]
#[owners_only]
async fn global(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut db = get_global_pickle_database("banned_words.db");

    db.set(args.rest(), &1)?;

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Banned Words List");
            let mut description = String::from("Added ");
            description.push_str(args.rest());
            description.push_str(" to the global word filter");
            e.description(description);
            e.footer(|f| {
                let mut footer = String::from("Requested by ");
                footer.push_str(&msg.author.name);
                f.text(footer);

                f
            });

            e
        });

        m
    }).await?;

    warn!("Added a global banned word: {}", args.rest());

    Ok(())
}

#[command]
#[description = "Clears *all* of a users strikes."]
#[usage("<User>")]
#[checks(Moderator)]
#[only_in(guilds)]
async fn clearstrikes(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let strikes = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let target = args.parse::<UserId>().unwrap();
    strikes.execute(
        "DELETE FROM strikes WHERE userid = (?1)",
        params![target.as_u64().to_string()],
    )?;
    let action = ModAction {
        target,
        moderator: msg.author.clone(),
        action_type: ModActionType::ClearStrikes,
        reason: None,
        details: None,
        guild: msg.guild_id.unwrap(),
    };
    log_mod_action(action, ctx);

    let target_user_name = target.to_user(&ctx.http).await.unwrap().name;

    match msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Moderation Subsystem");
            e.description(format!(
                "Cleared strikes for {}",
                target_user_name
            ));
            e.footer(|f| {
                f.text(format!("Requested by {}", &msg.author.name));

                f
            });
            e.colour(Colour::DARK_GREEN);
            e
        });

        m
    }).await {
        Err(err) => error!("Error sending clearstrike response: {:?}", err),
        Ok(_msg) => (),
    }

    Ok(())
}

#[command]
#[description = "Modifies a current strike"]
#[usage("<Case Number> <Thing to modify> <What to modify it to>")]
#[min_args(3)]
#[checks(Moderator)]
#[only_in(guilds)]
async fn modstrike(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let strikes = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let case_id = &args.single::<u32>()?;
    let modify_thing = &args.single::<String>().unwrap().to_lowercase();
    let new_value = args.rest();

    if modify_thing == "reason" {
        strikes.execute(
            "UPDATE strikes SET reason = ?1 WHERE id = ?2",
            params![new_value, case_id],
        )?;
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description(format!(
                    "Successfully modified case {}!",
                    case_id.to_string()
                ));
                e.field("Field", "Reason", true);
                e.field("New Value", new_value, true);
                e.colour(Colour::DARK_GREEN);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        }).await?;
    } else if modify_thing == "withdraw" {
        strikes.execute(
            "UPDATE strikes SET is_withdrawn = '1' WHERE id = ?1",
            params![case_id],
        )?;
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description(format!("Sucessfully withdrew case #{}", case_id));
                e.colour(Colour::DARK_GREEN);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        }).await?;
    } else {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description("You can only modify a strike's reason.");
                e.colour(Colour::RED);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        }).await?;
    }

    Ok(())
}

#[command]
#[usage("<Case Number>")]
#[num_args(1)]
#[checks(Moderator)]
#[only_in(guilds)]
async fn getstrike(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut user_id: UserId;
    let mut moderator_id: UserId;
    let mut reason: String;
    let mut is_withdrawn: bool;
    {
    let strikes = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let mut stmt =
        strikes.prepare("SELECT userid,moderator,reason,is_withdrawn FROM strikes WHERE id = ?")?;
    let mut rows = stmt.query(params![args.current()])?;
    let row = rows.next().unwrap().unwrap();
    user_id = row
        .get::<usize, String>(0)
        .unwrap()
        .parse::<u64>()
        .unwrap()
        .into();
    moderator_id = row
        .get::<usize, String>(1)
        .unwrap()
        .parse::<u64>()
        .unwrap()
        .into();
    reason = row.get::<usize, String>(2).unwrap().clone();
    
        is_withdrawn = match row.get::<usize, String>(3).unwrap().parse::<i32>().unwrap() {
            1 => true,
            _ => false,
        };

    }
    let user = match user_id.to_user(&ctx.http).await {
        Ok(u) => u,
        Err(err) => return Err(CommandError(err.to_string())),
    };

    let moderator = moderator_id.to_user(&ctx.http).await?;
    
    
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Moderation Case");
            e.description(reason);
            e.fields(vec![
                ("User", &user.name, true),
                ("Moderator", &moderator.name, true),
                ("Is Withdrawn?", &is_withdrawn.to_string(), true),
            ]);
            e.footer(|f| {
                f.text(format!("Requested by {}", &msg.author.name));
                f
            });
            e
        });
        m
    }).await;
    return Ok(());
}

#[command]
#[usage("<@Mention>")]
#[description = "Checks a user against the banlist and returns other information"]
#[num_args(1)]
#[only_in(guilds)]
async fn runuser(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db = get_discord_banlist();
    let age_db = get_global_pickle_database("age.db");
    let badge_db = get_badge_db();
    let target_id = match args.parse::<UserId>() {
        Ok(id) => id,
        Err(err) => {
            error!(
                "Error parsing userid for `runuser` command in {}: {:?}",
                &msg.guild_id.unwrap(),
                err
            );
            return Err(CommandError(err.to_string()));
        }
    };

    let mut badges: String = String::from("‎"); // Contains a unicode "blank space" to appease JSON
    let mut is_banned = false;

    let age_group = age_db.get::<String>(&target_id.as_u64().to_string());
    {
    let mut dbans_stmt = db
        .prepare("SELECT reason,guild_id,id FROM dbans WHERE userid = (?)")
        .unwrap();
    let mut ban_result = dbans_stmt.query(params![&target_id.as_u64().to_string()])?;
    if let Ok(o) = ban_result.next() {
        if let Some(_r) = o {
            is_banned = true;
        }
    }
}

    {
    // Badges
    let mut badge_stmt = badge_db.prepare("SELECT badge FROM badges WHERE userid = (?)")?;
    let mut badge_result = badge_stmt.query(params![&target_id.as_u64().to_string()])?;
    while let Some(r) = badge_result.next().unwrap() {
        let badge = match r.get::<usize, String>(0) {
            Ok(s) => s,
            Err(er) => return Err(CommandError(er.to_string())),
        };
        badges.push_str(&format!("{}\n", badge));
    }
}
    // Verified Roles
    //* Create the pickledb instance here, and then add it to the db_map later with the right key.
    let eagle_db = get_global_pickle_database("eagle.db");
    let summit_db = get_global_pickle_database("summit.db");
    let cstaff_db = get_global_pickle_database("campstaff.db");
    let ypt_db = get_global_pickle_database("ypt.db");
    let ordeal_db = get_global_pickle_database("ordeal.db");
    let brotherhood_db = get_global_pickle_database("brotherhood.db");
    let vigil_db = get_global_pickle_database("vigil.db");
    let qm_db = get_global_pickle_database("quartermaster.db");
    let mut db_map: HashMap<&str, pickledb::PickleDb> = HashMap::new();

    db_map.insert("Eagle", eagle_db);
    db_map.insert("Summit", summit_db);
    db_map.insert("Camp Staff", cstaff_db);
    db_map.insert("YPT", ypt_db);
    db_map.insert("Ordeal", ordeal_db);
    db_map.insert("Brotherhood", brotherhood_db);
    db_map.insert("Vigil", vigil_db);
    db_map.insert("Quartermaster", qm_db);

    let mut verified_roles = String::from("‎"); // Contains a unicode "blank space" to appease JSON
    for (key, db) in db_map {
        if let Some(i) = db.get::<i32>(&target_id.as_u64().to_string()) {
            verified_roles.push_str(&format!("{}\n", key));
            debug!("Found verified role {}", key);
        } else {
            debug!("Did not find verified role {}", key);
        }
    }

    let target_user = &ctx.http.get_user(*target_id.as_u64()).await?;
    let guild = &ctx.http.get_guild(*msg.guild_id.unwrap().as_u64()).await?;
    let user_name = &target_user.name;
    let user_id = target_id.as_u64();
    let member = guild.member(&ctx.http, *user_id).await?;
    let joined_guild_datetime = member.joined_at.unwrap();
    let joined_guild_date = joined_guild_datetime.date().naive_utc();
    let joined_guild_time = joined_guild_datetime.time();

    let joined_discord_datetime = target_id.created_at();
    let joined_discord_date = joined_discord_datetime.date().naive_utc();
    let joined_discord_time = joined_discord_datetime.time();

    let mut age_line = match age_group {
        Some(s) => s,
        None => String::from("Unknown Age"),
    };

    if age_line != String::from("Unknown Age") {
        age_line = format!("{} 18", age_line)
    }

    let user_avatar = match target_user.avatar_url() {
        Some(url) => url,
        None => target_user.default_avatar_url(),
    };

    if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("User Info");
            if is_banned {
                e.description("User has a current ban on a DSC member server.");
                e.colour(Colour::RED);
            } else {
                e.description("User is in good standing with DSC.");
                e.colour(Colour::DARK_GREEN);
            }
            e.thumbnail(user_avatar);
            e.fields(vec![
                (
                    "Name",
                    format!("{}#{}", user_name, target_user.discriminator),
                    true,
                ),
                ("ID", user_id.to_string(), true),
                (
                    "Joined Server",
                    format!("{}, {}Z", joined_guild_date, joined_guild_time),
                    true,
                ),
                (
                    "Joined Discord",
                    format!("{}, {}Z", joined_discord_date, joined_discord_time),
                    true,
                ),
                ("Age Group", age_line, true),
                ("Verified Roles", verified_roles, true),
                ("DSC Badges", badges, true),
            ]);

            e.footer(|f| {
                f.text(format!("DSC Bot | Powered by Rusty Developers"));
                f
            });

            e
        });
        m
    }).await {
        error!("Error sending `runuser` output: {:?}", err);
        return Err(CommandError(err.to_string()));
    }

    Ok(())
}

#[command]
#[description = "Sends the server's banlist to the DSC database"]
#[only_in(guilds)]
#[checks(Moderator)]
async fn syncbans(ctx: &Context, msg: &Message) -> CommandResult {
    let db = get_discord_banlist();
    let mut current_dsc_bans: Vec<DscBan> = Vec::new();
    {
    let mut stmt = db.prepare("SELECT userid,reason FROM dbans WHERE guild_id = ?1")?;
    debug!("Getting current bans");
    let res = stmt.query_map(
        params![&msg.guild_id.unwrap().as_u64().to_string()],
        |row| {
            Ok(DscBan {
                userid: row.get(0).unwrap(),
                reason: row.get(1).unwrap(),
            })
        },
    )?;
    for b in res {
        current_dsc_bans.push(b.unwrap());
    }
    }
    debug!("Getting guild bans");
    let guild_bans = &ctx.http.get_bans(*msg.guild_id.unwrap().as_u64()).await?;
    {
    let mut insert_stmt = db
        .prepare("INSERT INTO dbans(userid,reason,guild_id,is_withdrawn) VALUES (?1, ?2, ?3, 0)")?;

    debug!("Checking server bans against DSC bans");
    for b in guild_bans.iter() {
        let reason: String = match &b.reason {
            Some(r) => r.clone(),
            None => String::from("No reason provided"),
        };
        let b_userid = b.user.id.as_u64();
        let b_guildid = *msg.guild_id.unwrap().as_u64();
        if current_dsc_bans.len() > 0 {
            for dscb in current_dsc_bans.iter() {
                if !b.user.id.as_u64() == dscb.userid.parse::<u64>().unwrap() {
                    insert_stmt.execute(params![
                        b_userid.to_string(),
                        reason,
                        b_guildid.to_string()
                    ])?;
                }
            }
        } else {
            insert_stmt.execute(params![b_userid.to_string(), reason, b_guildid.to_string()])?;
        }
    }
}
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {let mut is_banned = false;
            e.description("Finished syncing bans to the DSC Banlist");
            e.colour(Colour::DARK_GREEN);
            e.footer(|f| {
                f.text(format!("DSC Bot | Powered by Rusty Development"));
                f
            });
            e
        });
        m
    }).await?;

    let guild = &ctx.http.get_guild(*msg.guild_id.unwrap().as_u64()).await.unwrap();
    info!("Synced bans from {}", &guild.name);

    debug!("Command finished");
    Ok(())
}

#[command]
#[description = "Sends an advisory on a user to DSC"]
#[only_in(guilds)]
#[usage("<User> <Reason>")]
#[min_args(2)]
#[checks(Moderator)]
async fn advise(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let target_user_id = match args.current().unwrap().parse::<UserId>() {
        Ok(u) => u,
        Err(err) => {
            error!(
                "Error parsing argument {} into a UserID: {:?}",
                args.current().unwrap(),
                err
            );
            return Err(CommandError(err.to_string()));
        }
    };
    args.advance();
    let reason = args.rest();
    let advise_channel = match &ctx.http.get_channel(646545388576178178).await {
        Ok(c) => c.id(),
        Err(err) => {
            error!("Error finding advisory channel: {:?}", err);
            return Err(CommandError(err.to_string()));
        }
    };

    let user_result = ctx.http.get_user(*target_user_id.as_u64()).await;

    let target_user = match &user_result {
        Ok(u) => u,
        Err(err) => {
            error!("Error fetching advisory target: {:?}", err);
            return Err(CommandError(err.to_string()));
        }
    };

    let guild_result = ctx.http.get_guild(*msg.guild_id.unwrap().as_u64()).await;

    let guild = match &guild_result {
        Ok(g) => g,
        Err(err) => {
            error!("Error fetching advising guild: {:?}", err);
            return Err(CommandError(err.to_string()));
        }
    };

    let avatar_url = match target_user.avatar_url() {
        Some(url) => url,
        None => target_user.default_avatar_url(),
    };

    match advise_channel.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("New Advisory Sent");
            e.fields(vec![
                ("User", target_user.name.clone(), false),
                ("Server", guild.name.clone(), false),
                ("Reason", String::from(reason), false),
            ]);
            e.color(Colour::ORANGE);
            e.thumbnail(avatar_url);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }).await {
        Err(err) => {
            error!("Error sending advisory message: {:?}", err);
            return Err(CommandError(err.to_string()));
        }
        _ => (),
    }

    match msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Advisory Sent");
            e.description("Dispatched your advisory to DSC.");
            e.colour(Colour::DARK_GREEN);
            e
        });
        m
    }).await {
        Err(err) => {
            error!("Error responding to message: {:?}", err);
            return Err(CommandError(err.to_string()));
        }
        _ => (),
    }

    Ok(())
}

#[command]
#[description = "Modifies a current strike"]
#[usage("<Case Number> <Thing to modify> <What to modify it to>")]
#[min_args(2)]
#[checks(VibeOfficer)]
#[only_in(guilds)]
async fn modban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let db = get_discord_banlist();
    let case_id = &args.single::<u32>()?;
    let modify_thing = &args.single::<String>().unwrap().to_lowercase();
    let new_value = args.rest();

    if modify_thing == "reason" {
        db.execute(
            "UPDATE dbans SET reason = ?1 WHERE id = ?2",
            params![new_value, case_id],
        )?;
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description(format!(
                    "Successfully modified case {}!",
                    case_id.to_string()
                ));
                e.field("Field", "Reason", true);
                e.field("New Value", new_value, true);
                e.colour(Colour::DARK_GREEN);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        }).await?;
    } else if modify_thing == "withdraw" {
        db.execute(
            "UPDATE dbans SET is_withdrawn = 1 WHERE id = ?1",
            params![case_id],
        )?;
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description(format!("Sucessfully withdrew case #{}", case_id));
                e.colour(Colour::DARK_GREEN);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        }).await?;
    } else {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description("You can only modify a strike's reason or withdraw.");
                e.colour(Colour::RED);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        }).await?;
    }

    Ok(())
}

#[command]
#[description = "Displays a list of strikes given to a user"]
#[only_in(guilds)]
#[min_args(1)]
#[checks(VibeOfficer)]
#[owner_privilege]
async fn bans(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let strike_conn = get_discord_banlist();
    let target_user = args.parse::<UserId>().unwrap();
    let mut bans: HashMap<StrikeLog, bool> = HashMap::new();
    {
    let mut stmt = strike_conn
        .prepare("SELECT reason,id,is_withdrawn FROM dbans WHERE userid = (?)")
        .unwrap();
    let mut rows = stmt
        .query(params![target_user.as_u64().to_string()])
        .unwrap();

    
    while let Some(row) = rows.next().unwrap() {
        let reason = row.get::<usize, String>(0);
        let case_id = row.get::<usize, u32>(1);
        let is_withdrawn = match row.get::<usize, u32>(2) {
            Ok(i) => {
                if i == 1 {
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        let strike = StrikeLog {
            user: target_user,
            moderator: UserId(705876821232844910),
            reason: reason.unwrap(),
            case_id: case_id.unwrap().to_string(),
        };
        bans.insert(strike, is_withdrawn);
    }
}
    let mut result_vec: Vec<(String, String, bool)> = Vec::new();

    for (_i, (s, w)) in bans.iter().enumerate() {
        if *w {
            result_vec.push((
                format!("Case #{}", s.case_id),
                format!("~~{}~~", s.reason.clone()),
                false,
            ));
        } else {
            result_vec.push((format!("Case #{}", s.case_id), s.reason.clone(), false));
        }
    }

    let target_user_name = target_user.to_user(&ctx.http).await.unwrap().name;

    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            let mut title = String::from("Bans for ");
            title.push_str(&target_user_name);
            e.title(title);

            e.fields(result_vec);

            let mut footer = String::from("Requested by ");
            footer.push_str(&msg.author.name);
            e.footer(|f| {
                f.text(footer);
                f
            });

            e
        });

        m
    }).await {
        Err(err) => error!("Error sending ban log: {:?}", err),
        Ok(_msg) => (),
    }

    Ok(())
}

#[command]
#[description = "Puts the server into raid mode and alerts DSC"]
#[only_in(guilds)]
#[checks(Moderator)]
#[num_args(0)]
async fn raid(ctx: &Context, msg: &Message) -> CommandResult {
    let http_cache = &ctx.http;
    let guild_arc = match msg.guild(&ctx).await {
        Some(a) => a,
        None => return Err(CommandError("Could not find message guild".to_string())),
    };
    let guild = guild_arc.read().await;
    let mut guild_channels = match guild.channels(http_cache).await {
        Ok(h) => h,
        Err(e) => return Err(CommandError(e.to_string())),
    };

    for (id, gc) in guild_channels.iter_mut() {
        match gc.kind {
            ChannelType::Text => {
                if let Err(e) = gc.edit(http_cache, |e| {
                    e.slow_mode_rate(21600);
                    e
                }).await {
                    error!("Error in raid: Could not set slowmode on channel {}: {:?}", id.as_u64().to_string(), e);
                }
            },
            _ => continue,
        }
    }
    if let Ok(c) = http_cache.get_channel(crate::prelude::NOTIFY_CHANNEL).await {
        if let Err(e) =c.id().send_message(&ctx, |m| {
            m.content("@everyone");
            m.embed(|e| {
                e.title("RAID IN PROGRESS");
                e.fields(vec![
                    ("Server", guild.name.clone(), true),
                    ("Moderator", format!("<@{}>", *msg.author.id.as_u64()), true),
                ]);
                e.footer(|f| {
                    f.text("DSC Bot | Powered by Rusty Development");
                    f
                });
                e.colour(Colour::RED);
                e
            });
            m
        }).await {
            return Err(CommandError(format!("Error alerting DSC to raid in {}: {:?}", guild.name.clone(), e.to_string())));
        }
    }
    if let Err(e) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Raid Mode");
            e.description("Successfully set raid mode on the server");
            e.colour(Colour::RED);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }).await {
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
#[description = "Takes the server out of raid mode and alerts DSC"]
#[only_in(guilds)]
#[checks(Moderator)]
#[num_args(0)]
async fn unraid(ctx: &Context, msg: &Message) -> CommandResult {
    let http_cache = &ctx.http;
    let guild_arc = match msg.guild(&ctx).await {
        Some(a) => a,
        None => return Err(CommandError("Could not find message guild".to_string())),
    };
    let guild = guild_arc.read().await;
    let mut guild_channels = match guild.channels(http_cache).await {
        Ok(h) => h,
        Err(e) => return Err(CommandError(e.to_string())),
    };

    for (id, gc) in guild_channels.iter_mut() {
        match gc.kind {
            ChannelType::Text => {
                if let Err(e) = gc.edit(http_cache, |e| {
                    e.slow_mode_rate(0);
                    e
                }).await {
                    error!("Error in unraid: Could not remove slowmode on channel {}: {:?}", id.as_u64().to_string(), e);
                }
            },
            _ => continue,
        }
    }
    if let Ok(c) = http_cache.get_channel(crate::prelude::NOTIFY_CHANNEL).await {
        if let Err(e) =c.id().send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Raid Mode Lifted");
                e.fields(vec![
                    ("Server", guild.name.clone(), true),
                    ("Moderator", format!("<@{}>", *msg.author.id.as_u64()), true),
                ]);
                e.footer(|f| {
                    f.text("DSC Bot | Powered by Rusty Development");
                    f
                });
                e.colour(Colour::BLUE);
                e
            });
            m
        }).await {
            return Err(CommandError(format!("Error alerting DSC to unraid in {}: {:?}", guild.name.clone(), e.to_string())));
        }
    }
    if let Err(e) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Raid Mode");
            e.description("Successfully removed raid mode on the server");
            e.colour(Colour::BLUE);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }).await {
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}