/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::framework::standard::{macros::command, CommandResult, StandardFramework, Args};
use serenity::{model::channel::Message, model::guild::Member, prelude::*};
use serenity::model::id::UserId;
use rusqlite::{Connection, params, Result};

use crate::checks::*;

struct Strike {
    user: UserId,
    reason: Option<String>
}

#[command]
#[description = "Adds a strike to the mentioned user"]
#[only_in(guilds)]
#[min_args(1)]
#[checks(Moderator)]
pub fn strike(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let strike_conn = Connection::open("strikes.db").unwrap();
    let strike = Strike {user: args.parse::<UserId>().unwrap(), reason: Some(String::from(args.advance().rest()))};
    strike_conn.execute("INSERT INTO strikes (userid, reason) VALUES (?1, ?2)", params![strike.user.as_u64().to_string(), strike.reason]).unwrap();

    msg.channel_id.say(&ctx.http, "Struck the user.").unwrap();

    Ok(())
}