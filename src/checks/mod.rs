use serenity::{
    framework::standard::{
        CheckResult, Args, CommandOptions, macros::check
    },
    model::channel::{Message}
};
use serenity::prelude::*;

#[check]
#[name = "Moderator"]
#[display_in_help]
pub fn mod_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {

        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.manage_guild().into();
        }

    }

    false.into()
}