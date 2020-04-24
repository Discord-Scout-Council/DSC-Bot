/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use super::data::get_global_pickle_database;
use serenity::{model::{user::User,id::{GuildId}, prelude::*}};
use serenity::client::Context;

pub enum ModActionType {
    Strike,
    BadWordDelete,
}

pub struct ModAction {
    pub target: UserId,
    pub moderator: User,
    pub action_type: ModActionType,
    pub reason: Option<String>,
    pub details: Option<String>,
    pub guild: GuildId

}

impl ModAction {
    
}

pub fn contains_banned_word(content: &String) -> bool{
    let mut db = get_global_pickle_database("banned_words.db");
    let mut banned_words = db.get_all();

    let mut lower_content = content.to_lowercase();

    for w in banned_words.iter() {
        if lower_content.contains(w) {
            return true;
        } else {
            continue;
        }
    }
    return false;
}

pub fn log_mod_action(action: ModAction, ctx: &mut Context) {
    let guild_id = &action.guild;
    let guild_arc = guild_id.to_guild_cached(&ctx).unwrap();
    let guild = guild_arc.read();
    let mod_log_channel = guild.channel_id_from_name(&ctx, "mod-logs");

    mod_log_channel.unwrap().send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Moderation Log Entry");
            e.fields(vec![
                ("User", &action.target.to_user(&ctx).unwrap().name, true),
                ("Moderator", &action.moderator.name, true)
            ]);

            if let Some(r) = &action.reason {
                e.field("Reason", r, true);
            } else {
                e.field("Reason", "No reason provided", true);
            }

            if let Some(d) = &action.reason {
                e.field("Details", d, true);
            }

            match &action.action_type {
                ModActionType::Strike => {
                    e.field("Type", "Strike", false);
            },
                ModActionType::BadWordDelete => {
                    e.field("Type", "Word Filter", false);
                }
            };

            e
        });

        m
    }).unwrap();
    

}