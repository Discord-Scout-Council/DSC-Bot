use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{model::channel::Message, prelude::*};

#[command]
#[description = "Pings the bot"]
pub fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.channel_id.say(&ctx.http, "Pong!") {
        println!("Err sending message: {}", err);
    };

    Ok(())
}

#[command]
#[description = "Provides helpful information about the bot"]
pub fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m | {
        m.embed(|e | {
            e.title("Campmaster Constantine");
            e.description("A Discord Bot for Camp Quarantine");
            e.field("Creator", "<@118455061222260736>", false);

            e
        });

        m
    });

    Ok(())
}