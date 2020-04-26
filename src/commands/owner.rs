/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::util::data::get_pickle_database;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{model::channel::Message, model::guild::Member, prelude::*};
use crate::data::init_guild_cache;

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

#[command]
#[description = "Initializes the Guild Cache"]
#[owners_only]
pub fn initcache(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut guild_cache = get_pickle_database(msg.guild_id.unwrap().as_u64(), &"cache.db");
    init_guild_cache(&mut guild_cache);
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Campmaster Constantine");
            e.description("Successfully initialized the Guild Cache");
            e.footer(|f| {
                let mut footer: String = String::from("Requested by ");
                footer.push_str(&msg.author.name);

                f
            });

            e
        });

        m
    })?;

    Ok(())
}
