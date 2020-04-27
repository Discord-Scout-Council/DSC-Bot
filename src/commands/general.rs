/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::utils::Colour;
use serenity::{model::channel::Message, model::guild::Member, prelude::*};

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
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Campmaster Constantine");
            e.description("A Discord Bot for Camp Quarantine");
            e.field("Creator", "<@118455061222260736>", true);
            e.field("Logo", "<@678816040146436117>", true);
            e.field("Git", "[git.sr.ht](https://git.sr.ht/~muirrum/campmasteronstantine)", false);
            e.field("Issues", "[todo.sr.ht](https://todo.sr.ht/~muirrum/Campmaster-Constantine)", true);
            e.field("Report an Issue or Suggestion", "cbotsuggest <Suggestion>", true);

            e.thumbnail("https://cdn.discordapp.com/attachments/697917247368462336/702621900022480986/image0.png");

            e
        });

        m
    })?;

    Ok(())
}

#[command]
#[description = "Displays information about the server"]
#[only_in(guilds)]
pub fn serverinfo(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_arc = msg.guild_id.unwrap().to_guild_cached(&ctx.cache).unwrap();
    let guild = guild_arc.read();
    let mut member_count = guild.member_count;

    let mut guild_owner = guild.owner_id.to_user(&ctx).unwrap().name;

    let icon_url = match guild.icon_url() {
        Some(url) => url,
        None => String::from("https://external-content.duckduckgo.com/iu/?u=http%3A%2F%2Fwww.meessendeclercq.be%2Fimages%2Fgallery%2Fartists%2FLDB_Image_Not_Found_web.jpg&f=1&nofb=1")
    };

    guild_owner.push_str("#");
    guild_owner.push_str(
        &guild
            .owner_id
            .to_user(&ctx)
            .unwrap()
            .discriminator
            .to_string(),
    );

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&guild.name);

            e.field("Member Count", member_count.to_string(), true);
            e.field("Server Owner", guild_owner, true);

            e.thumbnail(icon_url);

            e
        });

        m
    });

    Ok(())
}

#[command]
#[description = "Sends a suggestion to the bot developers"]
#[usage("<Suggestion>")]
#[min_args(1)]
pub fn botsuggest(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let suggest_channel = ctx.cache.read().guild_channel(662067035999698987).unwrap();
    let suggestion = args.rest();
    suggest_channel.read().send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Campmaster Suggestion");
            e.description(suggestion);
            e.field("Suggester", &msg.author.name, true);
            e.field("Guild", &msg.guild(&ctx).unwrap().read().name, true);

            e
        });

        m
    })?;

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Campmaster Suggestion");
            e.description("Successfully sent your suggestion!");
            e.colour(Colour::DARK_GREEN);

            e
        });

        m
    })?;

    Ok(())
}
