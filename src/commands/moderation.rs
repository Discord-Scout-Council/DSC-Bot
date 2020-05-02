/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use log::{error, debug, info, warn};
use rusqlite::{params};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::id::UserId;
use serenity::utils::Colour;
use serenity::{model::channel::Message, model::user::User, prelude::*};

use crate::checks::*;

use crate::util::{
    data::{
        get_discord_banlist, get_global_pickle_database, get_pickle_database, get_strike_database,
    },
    moderation::*,
};

struct Strike {
    user: UserId,
    reason: Option<String>,
    moderator: UserId,
}

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

#[command]
#[description = "Adds a strike to the mentioned user"]
#[only_in(guilds)]
#[usage("<@User> <Reason>")]
#[min_args(2)]
#[checks(Moderator)]
pub fn strike(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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

    msg.channel_id.say(&ctx.http, "Struck the user.").unwrap();
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
pub fn strikelog(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let strike_conn = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let target_user = args.parse::<UserId>().unwrap();

    let mut stmt = strike_conn
        .prepare("SELECT reason,moderator,id FROM strikes WHERE userid = (?)")
        .unwrap();
    let mut rows = stmt
        .query(params![target_user.as_u64().to_string()])
        .unwrap();

    let mut strikes: Vec<StrikeLog> = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let reason = row.get::<usize, String>(0);
        let moderator = row.get::<usize, String>(1);
        let case_id = row.get::<usize, u32>(2);
        let strike = StrikeLog {
            user: target_user,
            moderator: moderator.unwrap().parse::<u64>().unwrap().into(),
            reason: reason.unwrap(),
            case_id: case_id.unwrap().to_string(),
        };
        strikes.push(strike);
    }

    let mut result_vec: Vec<(String, String, bool)> = Vec::new();

    for (_i, r) in strikes.iter().enumerate() {
        result_vec.push((format!("Case #{}", r.case_id), r.reason.clone(), false));
    }

    match msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                let mut title = String::from("Strikes for ");
                title.push_str(&target_user.to_user(&ctx).unwrap().name);
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
        }) {
            Err(err) => error!("Error sending strike log: {:?}", err),
            Ok(_msg) => ()
        }

    Ok(())
}

#[command]
#[description = "Manages the bad words filter"]
#[sub_commands(add)]
pub fn wordfilter(_ctx: &mut Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Adds a word to the bad words list"]
#[checks(Moderator)]
#[sub_commands(global)]
pub fn add(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(&ctx, "Called word management")?;
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
            })?;
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
            })?;
        }
    }

    Ok(())
}

#[command]
#[description = "Adds a word to the global list"]
#[owners_only]
pub fn global(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
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
    })?;

    warn!("Added a global banned word: {}", args.rest());

    Ok(())
}

#[command]
#[description = "Clears *all* of a users strikes."]
#[usage("<User>")]
#[checks(Moderator)]
#[only_in(guilds)]
pub fn clearstrikes(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
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

    match msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Moderation Subsystem");
            e.description(format!(
                "Cleared strikes for {}",
                target.to_user(&ctx).unwrap().name
            ));
            e.footer(|f| {
                f.text(format!("Requested by {}", &msg.author.name));

                f
            });
            e.colour(Colour::DARK_GREEN);
            e
        });

        m
    }) {
        Err(err) => error!("Error sending clearstrike response: {:?}", err),
        Ok(_msg) => ()
    }

    Ok(())
}

#[command]
#[description = "Modifies a current strike"]
#[usage("<Case Number> <Thing to modify> <What to modify it to>")]
#[min_args(3)]
#[checks(Moderator)]
#[only_in(guilds)]
pub fn modstrike(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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
        })?;
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
        })?;
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
        })?;
    }

    Ok(())
}

#[command]
#[usage("<Case Number>")]
#[num_args(1)]
#[checks(Moderator)]
#[only_in(guilds)]
pub fn getcase(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let strikes = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let mut stmt =
        strikes.prepare("SELECT userid,moderator,reason,is_withdrawn FROM strikes WHERE id = ?")?;
    let mut rows = stmt.query(params![args.current()])?;
    let row = rows.next().unwrap().unwrap();
    let user_id: UserId = row
        .get::<usize, String>(0)
        .unwrap()
        .parse::<u64>()
        .unwrap()
        .into();
    let user: User = user_id.to_user(&ctx)?;
    let moderator_id: UserId = row
        .get::<usize, String>(1)
        .unwrap()
        .parse::<u64>()
        .unwrap()
        .into();
    let moderator = moderator_id.to_user(&ctx)?;
    let reason = row.get::<usize, String>(2).unwrap();
    let is_withdrawn = match row.get::<usize, String>(3).unwrap().parse::<i32>().unwrap() {
        1 => true,
        _ => false,
    };

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
    })?;

    Ok(())
}

#[command]
#[usage("<@Mention>")]
#[description = "Checks a user against the banlist and returns other information"]
#[num_args(1)]
#[only_in(guilds)]
pub fn runuser(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let db = get_discord_banlist();
    let target_id = args.parse::<UserId>().unwrap();
    let mut stmt = db
        .prepare("SELECT reason,guild_id,id FROM dbans WHERE userid = (?)")
        .unwrap();
    let mut ban_result = stmt.query(params![&target_id.as_u64().to_string()])?;
    let mut is_banned = false;
    if let Ok(o) = ban_result.next() {
        if let Some(_r) = o {
            is_banned = true;
        }
    }

    let target_user = &ctx.http.get_user(*target_id.as_u64())?;
    let guild = &ctx.http.get_guild(*msg.guild_id.unwrap().as_u64())?;
    let user_name = &target_user.name;
    let user_id = target_id.as_u64();
    let member = guild.member(&ctx, *user_id)?;
    let joined_guild_datetime = member.joined_at.unwrap();
    let joined_guild_date = joined_guild_datetime.date().naive_utc();
    let joined_guild_time = joined_guild_datetime.time();

    let joined_discord_datetime = target_id.created_at();
    let joined_discord_date = joined_discord_datetime.date().naive_utc();
    let joined_discord_time = joined_discord_datetime.time();

    let user_avatar = match target_user.avatar_url() {
        Some(url) => url,
        None => target_user.default_avatar_url(),
    };

    msg.channel_id.send_message(&ctx, |m| {
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
            ]);

            e.footer(|f| {
                f.text(format!("DSC Bot | Powered by Rusty Developers"));
                f
            });

            e
        });
        m
    })?;

    Ok(())
}


#[command]
#[description = "Sends the server's banlist to the DSC database"]
#[only_in(guilds)]
#[checks(Moderator)]
pub fn syncbans(ctx: &mut Context, msg: &Message) -> CommandResult {
    let db = get_discord_banlist();
    let mut stmt = db.prepare("SELECT userid,reason FROM dbans WHERE guild_id = ?1")?;
    debug!("Getting current bans");
    let res = stmt.query_map(params![&msg.guild_id.unwrap().as_u64().to_string()], |row| {
        Ok(DscBan {
            userid: row.get(0).unwrap(),
            reason: row.get(1).unwrap(),
        })
    })?;
    let mut current_dsc_bans: Vec<DscBan> = Vec::new();
    for b in res {
        current_dsc_bans.push(b.unwrap());
    }

    debug!("Getting guild bans");
    let guild_bans = &ctx.http.get_bans(*msg.guild_id.unwrap().as_u64())?;

    let mut insert_stmt = db.prepare("INSERT INTO dbans(userid,reason,guild_id,is_withdrawn) VALUES (?1, ?2, ?3, 0)")?;

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


                insert_stmt.execute(params![b_userid.to_string(),reason,b_guildid.to_string()])?;
            }
        }
    } else {
        insert_stmt.execute(params![b_userid.to_string(),reason,b_guildid.to_string()])?;
    }
    }
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("DSC Banlist");
            e.description("Finished syncing bans to the DSC Banlist");
            e.colour(Colour::DARK_GREEN);
            e.footer(|f| {
                f.text(format!("DSC Bot | Powered by Rusty Development"));
                f
            });
            e
        });
        m
    })?;

    let guild = &ctx.http.get_guild(*msg.guild_id.unwrap().as_u64()).unwrap();
    info!("Synced bans from {}", &guild.name);

    debug!("Command finished");
    Ok(())
}