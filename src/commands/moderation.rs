/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use rusqlite::{params, Connection, Result, RowIndex};
use serenity::framework::standard::{macros::command, Args, CommandResult, StandardFramework};
use serenity::model::id::UserId;
use serenity::utils::Colour;
use serenity::{model::channel::Message, model::user::User, prelude::*};

use crate::checks::*;

use crate::util::{
    data::{get_global_pickle_database, get_pickle_database, get_strike_database},
    moderation::*,
};

struct Strike {
    user: UserId,
    reason: Option<String>,
    moderator: UserId
}

struct StrikeLog {
    user: UserId,
    reason: String,
    moderator: UserId,
    case_id: String,
    is_withdrawn: u32
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
            "INSERT INTO strikes (userid, reason, moderator, is_withdrawn) VALUES (?1, ?2, ?3, ?4)",
            params![strike.user.as_u64().to_string(), strike.reason, strike.moderator.as_u64().to_string(), "0"],
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
pub fn strikelog(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let strike_conn = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let target_user = args.parse::<UserId>().unwrap();

    let mut stmt = strike_conn
        .prepare("SELECT reason,moderator,id,is_withdrawn FROM strikes WHERE userid = (?)")
        .unwrap();
    let mut rows = stmt
        .query(params![target_user.as_u64().to_string()])
        .unwrap();

    let mut strikes: Vec<StrikeLog> = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let reason = row.get::<usize, String>(0);
        let moderator = row.get::<usize, String>(1);
        let case_id = row.get::<usize, u32>(2);
        let is_withdrawn = row.get::<usize, String>(3);
        let strike = StrikeLog {
            user: target_user,
            moderator: moderator.unwrap().parse::<u64>().unwrap().into(),
            reason: reason.unwrap(),
            case_id: case_id.unwrap().to_string(),
            is_withdrawn: is_withdrawn.unwrap().parse::<u32>().unwrap(),
        };
        strikes.push(strike);
    }

    let mut result_vec: Vec<(String, String, bool)> = Vec::new();

    for (i, r) in strikes.iter().enumerate() {
        if r.is_withdrawn == 1 {
            result_vec.push((format!("Case #{}", r.case_id), format!("~~{}~~", r.reason.clone()), false));
        } else {
            result_vec.push((format!("Case #{}",r.case_id), r.reason.clone(), false));
        }
    }

    msg.channel_id
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
        })
        .unwrap();

    Ok(())
}

#[command]
#[description = "Manages the bad words filter"]
#[sub_commands(add)]
pub fn wordfilter(ctx: &mut Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
#[description = "Adds a word to the bad words list"]
#[checks(Moderator)]
#[sub_commands(global)]
pub fn add(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.reply(&ctx, "Called word management")?;
    let guild = &msg.guild_id.unwrap();
    let mut db = get_pickle_database(guild.as_u64(), "banned_words.db");
    match db.get::<i32>(&args.rest()) {
        Some(i) => {
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
            db.set(&args.rest(), &1)?;
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
pub fn global(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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

    Ok(())
}

#[command]
#[description = "Clears *all* of a users strikes."]
#[usage("<User>")]
#[checks(Moderator)]
#[only_in(guilds)]
pub fn clearstrikes(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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

    msg.channel_id.send_message(&ctx, |m| {
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
    })?;

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
        strikes.execute("UPDATE strikes SET reason = ?1 WHERE id = ?2", params![new_value, case_id])?;
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation");
                e.description(format!("Successfully modified case {}!", case_id.to_string()));
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
        strikes.execute("UPDATE strikes SET is_withdrawn = '1' WHERE id = ?1", params![case_id])?;
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
    } 
    else {
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
pub fn getcase(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let strikes = get_strike_database(&msg.guild_id.unwrap().as_u64());
    let mut stmt = strikes.prepare("SELECT userid,moderator,reason,is_withdrawn FROM strikes WHERE id = ?")?;
    let mut rows = stmt.query(params![args.current()])?;
    let row = rows.next().unwrap().unwrap();
    let user_id: UserId = row.get::<usize, String>(0).unwrap().parse::<u64>().unwrap().into();
    let user: User = user_id.to_user(&ctx)?;
    let moderator_id: UserId = row.get::<usize, String>(1).unwrap().parse::<u64>().unwrap().into();
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
                ("Is Withdrawn?", &is_withdrawn.to_string(), true)
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