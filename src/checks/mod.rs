/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::check, Args, CheckResult, CommandOptions},
    model::{
        channel::Message,
        id::{GuildId, RoleId},
    },
};

#[check]
#[name = "Moderator"]
#[display_in_help]
async fn mod_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache).await {
        if let Ok(permissions) = member.permissions(&ctx.cache).await {
            return permissions.manage_guild().into();
        }
    }

    false.into()
}

#[check]
#[name = "VibeOfficer"]
#[display_in_help]
async fn vibe_check(
    ctx: &Context,
    msg: &Message,
) -> CheckResult {
    let user = &msg.author;
    let vibe_role_id = RoleId(699802594750759043);
    let vibe_guild_id = GuildId(646540745443901469);
    let foreman_role = RoleId(660494289171906580);
    let http_cache = &ctx.http;
    let member = match &http_cache.get_member(*vibe_guild_id.as_u64(), *user.id.as_u64()).await {
        Ok(m) => m.clone(),
        Err(err) => return CheckResult::new_log("Could not fetch member"),
    };
    let member_roles = &member.roles;
    for r in member_roles.iter() {
        if r.as_u64() == vibe_role_id.as_u64() || r.as_u64() == foreman_role.as_u64() {
            return CheckResult::Success;
        }
    }

    return CheckResult::new_unknown();
}
