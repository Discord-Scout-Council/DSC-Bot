/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::prelude::*;
use serenity::model::channel::{Reaction, ReactionType};

enum VerifyType {
    Eagle,
    SummitSilver,
    CampStaff,
    Ypt,
    Ordeal,
    Brotherhood,
    Vigil,
    Close,
}

pub fn handle_verification_file(ctx: &Context, msg: &Message) -> Result<(), String> {
    if msg.attachments.len() == 0 {
        return Err(String::from("No attachments found"));
    }

    let http_cache = &ctx.http;

    let verify_channel = match http_cache.get_channel(684577265425973285) {
        Ok(c) => c,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let verify_channel_id = verify_channel.id();

    let verify_message = match verify_channel_id.send_message(&ctx, |m| {
        m.content(format!(
            "{}\n{}",
            &msg.author.id.as_u64().to_string(),
            msg.attachments.get(0).unwrap().url
        ));
        m
    }) {
        Err(err) => return Err(err.to_string()),
        Ok(m) => m,
    };

    let emoji_vec = define_emoji_vec();

    for s in emoji_vec {
        if let Err(err) = verify_message.react(http_cache, s) {
            return Err(err.to_string());
        }
    }

    Ok(())
}

pub fn handle_verification_reaction(ctx: &Context, react: Reaction) -> Result<String, String> {
    let current_info = &ctx.http.get_current_application_info().unwrap();
    if react.user_id.as_u64() == current_info.id.as_u64() {
        return Ok(String::from(""));
    }
    let message_id = react.message_id;
    let http_cache = &ctx.http;
    let message = match http_cache.get_message(684577265425973285, *message_id.as_u64()) {
        Ok(m) => m,
        Err(err) => return Err(err.to_string()),
    };

    let emoji_used = match react.emoji {
        ReactionType::Unicode(e) => e,
        _ => {
            return Err(String::from("Invalid reaction"));
        }
    };

    let emoji_vec = define_emoji_vec();

    let verify_type = match match_verify_type(&emoji_used) {
        Some(v) => v,
        None => return Err(String::from("Invalid verify type")),
    };

    let message_content = &message.content;
    let user_id = message_content.split("\n").clone();
    let split_contents = user_id.collect::<Vec<&str>>();
    let user_id_str: &str = split_contents.get(0).unwrap();

    let user = match http_cache.get_user(split_contents.get(0).unwrap().parse::<u64>().unwrap()) {
        Err(err) => return Err(err.to_string()),
        Ok(u) => u,
    };

    let priv_chan = match user.create_dm_channel(http_cache) {
        Ok(c) => c,
        Err(err) => return Err(err.to_string()),
    };

    let mut verify_db = match verify_type {
        VerifyType::Eagle => get_global_pickle_database("eagle.db"),
        VerifyType::SummitSilver => get_global_pickle_database("summit.db"),
        VerifyType::CampStaff => get_global_pickle_database("campstaff.db"),
        VerifyType::Ypt => get_global_pickle_database("ypt.db"),
        VerifyType::Ordeal => get_global_pickle_database("ordeal.db"),
        VerifyType::Brotherhood => get_global_pickle_database("brotherhood.db"),
        VerifyType::Vigil => get_global_pickle_database("vigil.db"),
        VerifyType::Close => {
            if let Err(err) = priv_chan.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Verification");
                    e.description("Verification Request Closed.");
                    e.colour(Colour::RED);
                    e.footer(|f| {
                        f.text("DSC Bot | Powered by Rusty Development");
                        f
                    });
                    e
                });
                m
            }) {
                return Err(format!("Could not send closed request message to {}: {:?}", user_id_str.to_string(), err));
            }
            if let Err(err) = message.delete(http_cache) {
                return Err(format!("Could not delete verification message: {:?}",err.to_string()));
            }
            return Ok(String::from("Request closed"));
        }
    };

    if let Err(err) = verify_db.set(&user_id_str, &1) {
        return Err(err.to_string());
    }

    if let Err(err) = priv_chan.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Verification Request Status Update");
            e.description("Successfully verified.");
            e.colour(Colour::DARK_GREEN);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }) {
        return Err(format!("Error sending verification success message {:?}", err.to_string()));
    }
    if let Err(err) = message.delete(http_cache) {
        return Err(format!("Failed to delete verification request message: {:?}", err.to_string()));
    }

    Ok(String::from(""))
}

fn define_emoji_vec<'a>() -> Vec<&'a str> {
    let emoji_vec = vec!["🦅", "⛰", "🏕", "🛂", "↗", "🟥", "🔺", "❌", "⚠", "⛔"];

    emoji_vec
}

fn match_verify_type(emoji_used: &str) -> Option<VerifyType> {
    let emoji_vec = define_emoji_vec();
    match emoji_used {
        "🦅" => return Some(VerifyType::Eagle),
        "⛰" => return Some(VerifyType::SummitSilver),
        "🏕" => return Some(VerifyType::CampStaff),
        "🛂" => return Some(VerifyType::Ypt),
        "↗" => return Some(VerifyType::Ordeal),
        "🟥" => return Some(VerifyType::Brotherhood),
        "🔺" => return Some(VerifyType::Vigil),
        "❌" => return Some(VerifyType::Close),
        _ => return None,
    }
}
