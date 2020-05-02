/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::checks::*;
use crate::prelude::*;
use crate::util::data::{get_pickle_database, init_guild_settings};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::id::{ChannelId, RoleId};
use serenity::utils::Colour;
use serenity::{model::channel::Message, prelude::*};

#[command]
#[description = "Manage server settings"]
#[checks(Moderator)]
#[sub_commands(get, set)]
#[only_in(guilds)]
pub fn serversettings(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Server Settings Help");
            e.description("Changes and views server settings for the current server\n\nUse a value from the table below to change or view a setting");
            e.fields(vec![
                ("Usage", "serversettings set/get <setting> (value)", false),
                ("QOTD Channel", "qotd_channel", true),
                ("QOTD Role", "qotd_role", true),
                ("QOTD Suggestions Channel", "qotd_suggest_channel", true),
                ("Modlogs Channel", "modlogs_channel", true)
            ]);
            e.footer(|f| {
                f.text(format!("Requested by {}", &msg.author.name));
                f
            });
            e
        });
        m
    }) {
        error!("Error sending server settings help: {:?}", err);
    }

    Ok(())
}

#[command]
#[description = "Gets current value of a setting"]
#[checks(Moderator)]
#[num_args(1)]
pub fn get(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let db = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "settings.db");

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
        }
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
#[description = "Sets a setting"]
#[usage("<Setting> <Value>")]
#[min_args(2)]
pub fn set(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut settings = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "settings.db");
    let setting_name = args.current().unwrap();
    let mut arg_value = args.clone();
    let setting_value = if setting_name.to_lowercase().contains("role") {
        arg_value
            .advance()
            .rest()
            .parse::<RoleId>()
            .unwrap()
            .as_u64()
            .clone()
    } else if setting_name.to_lowercase().contains("channel") {
        let channel = &arg_value.advance().rest().parse::<ChannelId>().unwrap();
        channel.as_u64().clone()
    } else {
        0u64
    };

    if setting_value == 0 {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Server Settings");
                e.description(format!("Invalid setting value."));

                e
            });

            m
        })?;
    }

    if let None = settings.get::<u64>(&setting_name) {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Server Settings");
                e.description(format!(
                    "Setting {} does not exist. Refer to command help",
                    setting_name
                ));
                e.colour(Colour::RED);
                e
            });
            m
        })?;
    } else {
        let old_value = settings.get::<u64>(&setting_name).unwrap();
        settings.set(setting_name, &setting_value)?;
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Server Settings");
                e.description("Successfully changed setting");
                e.field("Setting", setting_name, true);
                e.field("New Value", setting_value, true);
                e.field("Old Value", old_value, false);
                e.colour(Colour::DARK_GREEN);
                e.footer(|f| {
                    f.text(format!("Requested by {}", &msg.author.name));
                    f
                });
                e
            });
            m
        })?;
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
