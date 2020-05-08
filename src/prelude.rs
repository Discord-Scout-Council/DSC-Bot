/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

pub const NOTIFY_CHANNEL: u64 = 646545388576178178u64;
pub const NOTIFY_GUILD: u64 = 646540745443901469;

pub use crate::util::data::*;
pub use log::{debug, error, info, warn};
pub use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
pub use serenity::model::id::UserId;
pub use serenity::utils::Colour;
pub use serenity::{model::channel::Message, model::guild::Member, prelude::*};
