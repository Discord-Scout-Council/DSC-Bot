use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{model::channel::Message, prelude::*};

use crate::util::parse_args;

pub struct Command {
    pub key: String,
    pub description: String,
}

pub fn define_commands() -> Vec<Command> {
    let ping = Command {
        key: String::from("!ping"),
        description: String::from("Pings the bot"),
    };

    vec![ping]
}

#[command]
pub fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.channel_id.say(&ctx.http, "Pong!") {
        println!("Err sending message: {}", err);
    };

    Ok(())
}

