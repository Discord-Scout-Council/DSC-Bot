/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::util::data::get_pickle_database;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::{model::channel::Message, prelude::*};

use crate::prelude::*;

#[command]
#[description = "Restarts the bot"]
#[owners_only]
async fn restart(ctx: &Context, msg: &Message) -> CommandResult {
    match msg
        .channel_id
        .say(&ctx.http, "Restarting bot, and applying new changes").await
    {
        Err(err) => error!("Error sending restart response: {:?}", err),
        Ok(_msg) => (),
    }
    warn!("{} is restarting the bot!", &msg.author.name);
    std::process::exit(0);
}

#[command]
#[description = "Initializes the Guild Cache"]
#[owners_only]
async fn initcache(ctx: &Context, msg: &Message) -> CommandResult {
    let mut guild_cache = get_pickle_database(msg.guild_id.unwrap().as_u64(), &"cache.db");
    if let Err(err) = guild_cache.set("current_qotd", &0) {
        error!(
            "Error setting current_qotd in {} cache: {:?}",
            msg.guild_id.unwrap().as_u64(),
            err
        );
    }
    if let Err(err) = msg.channel_id.send_message(&ctx.http, |m| {
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
    }).await {
        error!(
            "Error responding to {} cache init: {:?}",
            msg.guild_id.unwrap().as_u64(),
            err
        );
    }

    warn!(
        "{} just initialized the guild cache for guild {}",
        &msg.author.name,
        &msg.guild_id.unwrap().as_u64()
    );

    Ok(())
}
