/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use rusqlite::{params, Connection, Result};
use serenity::framework::standard::{macros::command, Args, CommandResult, StandardFramework};
use serenity::model::id::UserId;
use serenity::{model::channel::Message, model::guild::Member, prelude::*};

use crate::checks::*;

use crate::util::data::get_strike_database;

struct Strike {
    user: UserId,
    reason: Option<String>,
}

#[command]
#[description = "Adds a strike to the mentioned user"]
#[only_in(guilds)]
#[min_args(1)]
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
