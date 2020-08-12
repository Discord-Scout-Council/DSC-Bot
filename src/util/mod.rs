/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */
pub mod data;
pub mod moderation;
pub mod verification;
use crate::prelude::*;
use serenity::model::{channel::ReactionType, id::ChannelId};
use std::error::Error;

pub async fn run_yesno_vote(
    ctx: &Context,
    channel: ChannelId,
    vote: String,
) -> Result<(), Box<dyn Error>> {
    let msg: Message = match channel
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Vote");
                e.description(vote);
                e
            });
            m
        })
        .await
    {
        Ok(msg) => msg,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    if let Err(err) = msg
        .react(&ctx.http, ReactionType::Unicode("☑️".to_string()))
        .await
    {
        return Err(Box::new(err));
    }

    if let Err(err) = msg
        .react(&ctx.http, ReactionType::Unicode("❌".to_string()))
        .await
    {
        return Err(Box::new(err));
    }

    Ok(())
}
