
use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{StandardFramework, CommandResult, macros::command};

pub struct Command {
    pub key: String,
    pub description: String
}

pub fn define_commands() -> Vec<Command>{
    let ping = Command{key: String::from("!ping"), description: String::from("Pings the bot")};

    vec![ping]
}

#[command]
pub fn ping(ctx: &mut Context, msg: &Message) -> CommandResult{
    if let Err(err) = msg.channel_id.say(&ctx.http, "Pong!") {
        println!("Err sending message: {}", err);
    };

    Ok(())
}

#[command]
pub fn kick(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Called kick command");
    Ok(())
}
