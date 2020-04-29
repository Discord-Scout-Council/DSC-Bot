/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::util::{data::get_pickle_database, leveling::*};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::utils::Colour;
use serenity::{model::channel::Message, model::guild::Member, prelude::*};

const title: &str = "Leveling";

#[command]
#[description = "Gets your current level and shows how many points you have to earn to reach the next level."]
#[only_in(guilds)]
pub fn level(ctx: &mut Context, msg: &Message) -> CommandResult {
    let points_db = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "points.db");
    let points = match points_db.get::<u64>(&msg.author.id.as_u64().to_string()) {
        Some(p) => p,
        None => 0,
    };
    let level = get_level_from_points(points);
    let mut cost_to_next_level = get_level_cost(level + 1);
    cost_to_next_level = cost_to_next_level - points;
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title(title);
            e.fields(vec![
                ("Current Level", level.to_string(), true),
                ("Points to next level", cost_to_next_level.to_string(), true),
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
