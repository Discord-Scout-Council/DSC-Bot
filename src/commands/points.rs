/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{model::channel::Message, prelude::*};
use pickledb::*;

#[command]
#[description = "See how many points you have earned by chatting"]
#[only_in(guilds)]
pub fn points(ctx: &mut Context, msg: &Message) -> CommandResult {

    let db = PickleDb::load_yaml("points.db", PickleDbDumpPolicy::AutoDump).unwrap();
    let author_id = msg.author.id.as_u64();

    let points = match db.get(&author_id.to_string()) {
        Some(points) => points,
        None => 0,
    };

    msg.channel_id.send_message(&ctx.http, |m | {
        m.embed(|e| {

            let mut title = msg.author.name.clone();
            title.push_str("'s profile");

            e.title(title);
            e.field("Points:", points.to_string(), false);

            e
        });

        m
    });

    Ok(())
}