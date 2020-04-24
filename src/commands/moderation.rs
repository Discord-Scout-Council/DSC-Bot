/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use rusqlite::{params, Connection, Result, RowIndex};
use serenity::framework::standard::{macros::command, Args, CommandResult, StandardFramework};
use serenity::model::id::UserId;
use serenity::{model::channel::Message, model::guild::Member, prelude::*};
use serenity::utils::Colour;

use crate::checks::*;

use crate::util::{data::{get_strike_database, get_global_pickle_database, get_pickle_database}, moderation::*};

struct Strike {
    user: UserId,
    reason: Option<String>,
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
    };
    strike_conn
        .execute(
            "INSERT INTO strikes (userid, reason) VALUES (?1, ?2)",
            params![strike.user.as_u64().to_string(), strike.reason],
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
        .prepare("SELECT reason FROM strikes WHERE userid = (?)")
        .unwrap();
    let mut rows = stmt
        .query(params![target_user.as_u64().to_string()])
        .unwrap();

    let mut reasons: Vec<String> = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        reasons.push(row.get(0)?);
    }

    let mut result_vec: Vec<(usize, String, bool)> = Vec::new();

    for (i, r) in reasons.iter().enumerate() {
        result_vec.push((i + 1, r.clone(), false));
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
        },
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
pub fn global (ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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
