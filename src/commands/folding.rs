/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::prelude::*;
use std::io::Read;
use serenity::{
    http::GuildPagination,
    model::{
        guild::PartialGuild,
        id::GuildId,
        invite::{Invite, RichInvite},
    },
};

#[command]
#[description = "Provides F@H Leaderboard for the DSC F@H Team."]
pub fn fahleaderboard(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        let mut res = reqwest::get("https://api.foldingathome.org/team/262889/members")?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        m.embed(|e| {
            e.title("DSC F@H Stats (Points)");
            
            for token in body.split("\"],[\""){
                let tokens:Vec<&str>= token.split("\",\"").collect();
                    e.field(tokens[0], format!("Rank: {} Points: {} WUs: {}", tokens[2],tokens[3],tokens[4]), False);
            }

            e.thumbnail("https://apps.foldingathome.org/awards?team=262889");

            e
        });
        m
    })?;

    Ok(())
}

#[command]
#[description = "Provides F@H Stats for the DSC F@H Team."]
pub fn fahteamstats(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        let mut res = reqwest::get("https://api.foldingathome.org/team/262889")?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        let tokens:Vec<&str>= token.split(",").collect();
        m.embed(|e| {
            e.title("DSC F@H Stats");
            e.field("Total WUs", tokens[6], true);
            e.field("Total Points", tokens[5], true);
            e.field("Team Rank", tokens[7], true);

            e.thumbnail("https://apps.foldingathome.org/awards?team=262889&type=wus");

            e
        });

        m
    })?;

    Ok(())
}

