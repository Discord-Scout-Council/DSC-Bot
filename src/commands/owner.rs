/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{model::channel::Message, model::guild::Member, prelude::*};

#[command]
#[description = "Restarts the bot"]
#[owners_only]
pub fn restart(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Restarting bot, and applying new changes");
    ctx.shard.shutdown_clean();
    std::process::exit(0);
    Ok(())
}
