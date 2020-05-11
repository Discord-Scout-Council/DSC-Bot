/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use super::data::{get_global_pickle_database, get_pickle_database};
use serenity::client::Context;
use serenity::model::{id::GuildId, prelude::*, user::User};

pub enum ModActionType {
    Strike,
    BadWordDelete,
    ClearStrikes,
}

pub struct ModAction {
    pub target: UserId,
    pub moderator: User,
    pub action_type: ModActionType,
    pub reason: Option<String>,
    pub details: Option<String>,
    pub guild: GuildId,
}

impl ModAction {}

pub fn contains_banned_word(content: &String, guild_id: &u64) -> bool {
    let mut global_db = get_global_pickle_database("banned_words.db");
    let local_db = get_pickle_database(guild_id, "banned_words.db");
    let mut banned_words = global_db.get_all();
    let local_banned_words = local_db.get_all();
    for w in local_banned_words.iter() {
        banned_words.push(w.clone());
    }

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

async fn log_mod_action(action: ModAction, ctx: &mut Context) {
    let guild_id = &action.guild;
    let settings = get_pickle_database(guild_id.as_u64(), "settings.db");
    let mod_log_channel: ChannelId = settings.get::<u64>("modlogs_channel").unwrap().into();

    let target_name: &String = &action.target.to_user(&ctx.http).await.unwrap().name;

    mod_log_channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Moderation Log Entry");
                e.fields(vec![
                    ("User", &target_name, true),
                    ("Moderator", &&action.moderator.name, true),
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
                    }
                    ModActionType::BadWordDelete => {
                        e.field("Type", "Word Filter", false);
                    }
                    ModActionType::ClearStrikes => {
                        e.field("Type", "Strikelog Clear", false);
                    }
                };

                e
            });

            m
        })
        .await
        .unwrap();
}
