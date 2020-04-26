/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

 use crate::checks::*;
 use crate::util::data::get_pickle_database;
 use pickledb::{PickleDb, PickleDbDumpPolicy};
 use serenity::framework::standard::{macros::command, Args, CommandResult, StandardFramework};
 use serenity::model::id::UserId;
 use serenity::utils::Colour;
 use serenity::{model::channel::Message, model::guild::Member, prelude::*};
 use std::cmp::Ordering;

 #[command]
 #[description = "Manage server settings"]
 #[checks(Moderator)]
 #[sub_commands(get)]
 pub fn serversettings(ctx: &mut Context, msg: &Message) -> CommandResult {

    msg.reply(&ctx, "Help is a work in progress, like this command. Ping Muirrum for help!")?;

    Ok(())
}

#[command]
#[description = "Gets current value of a setting"]
#[checks(Moderator)]
#[num_args(1)]
pub fn get(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let db = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "cache.db");

    match db.get::<String>(&args.rest()) {
        Some(s) => {
            msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Server Settings");
                    e.field("Setting", &args.rest(), true);
                    e.field("Value", s, true);
                    e.colour(Colour::DARK_GREEN);

                    e
                });

                m
            })?;
        },
        None => {
            msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Server Settings");
                    let mut description = String::from("");
                    description.push_str("Could not find that setting. Check your spelling and try again\n");
                    description.push_str("\nIf you believe that this setting *should* exist, try running `cinitsettings` to get the default server settings initialized.");
                    e.description(description);
                    e.colour(Colour::RED);

                    e
                });

                m
            })?;
        }
    }

    Ok(())
}
#[command]
#[description = "Resets server settings"]
#[checks(Moderator)]
pub fn resetsettings(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut db = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "settings.db");

    init_guild_settings(&mut db);

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Server Settings");
            e.description(format!("Successfully reset server settings"));
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