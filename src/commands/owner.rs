/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::util::data::{get_pickle_database, init_guild_settings};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{model::channel::Message, model::guild::Member, prelude::*};

use log::warn;

#[command]
#[description = "Restarts the bot"]
#[owners_only]
pub fn restart(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Restarting bot, and applying new changes");
    warn!("{} is restarting the bot!", &msg.author.name);
    ctx.shard.shutdown_clean();
    std::process::exit(0);
    Ok(())
}

#[command]
#[description = "Initializes the Guild Cache"]
#[owners_only]
pub fn initcache(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut guild_cache = get_pickle_database(msg.guild_id.unwrap().as_u64(), &"cache.db");
    guild_cache.set("current_qotd", &0);
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

    warn!("{} just initialized the guild cache for guild {}", &msg.author.name, &msg.guild_id.unwrap().as_u64());

    Ok(())
}
