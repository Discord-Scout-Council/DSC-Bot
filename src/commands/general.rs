/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::prelude::*;

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
            e.title("DSC Bot");
            e.description("A Discord Bot for Discord Scout Council");
            e.field("Creator", "<@118455061222260736>", true);
            e.field("Report an Issue or Suggestion", "cbotsuggest <Suggestion>", true);

            e.thumbnail("https://cdn.discordapp.com/attachments/705877153513865328/705877361304010793/DSC_Logo.png");

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
    let member_count = guild.member_count;

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

    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(&guild.name);

            e.field("Member Count", member_count.to_string(), true);
            e.field("Server Owner", guild_owner, true);

            e.thumbnail(icon_url);

            e
        });

        m
    }) {
        Err(err) => error!("Error sending server count: {:?}", err),
        Ok(_msg) => ()
    }

    Ok(())
}

#[command]
#[description = "Sends a suggestion to the bot developers"]
#[usage("<Suggestion>")]
#[min_args(1)]
pub fn botsuggest(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let suggest_channel = ctx.cache.read().guild_channel(668964814684422184).unwrap();
    let suggestion = args.rest();
    suggest_channel.read().send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Bot Suggestion");
            e.description(suggestion);
            e.field("Suggester", &msg.author.name, true);
            e.field("Guild", &msg.guild(&ctx).unwrap().read().name, true);

            e
        });

        m
    })?;

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Bot Suggestion");
            e.description("Successfully sent your suggestion!");
            e.colour(Colour::DARK_GREEN);

            e
        });

        m
    })?;

    Ok(())
}

#[command]
#[description = "Describes what kind of data the bot collects, and what you can do to get your data removed."]
pub fn privacy(ctx: &mut Context, msg: &Message) -> CommandResult {
    match msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Privacy");
            e.description("The DSC bot collects the bare minimum data necessary to function. This data is only stored when necessary to carry out the primary functions of the bot, like notifying other servers of bans or advisories. When you are banned from a server or an advisory is put out about you, the bot collects and stores your UserID, the server the action was sent from, and the reason behind it (if any). Additionally, the bot may store your age, and any verified BSA awards or advancements that you choose to store. If you wish to know what information the bot is storing or to remove your information, please contact DSC.");
            e.field("Information Contact", "[support@devosmium.xyz](mailto:support@devosmium.xyz)", true);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });

            e
        });
        m
    }) {
        Err(err) => error!("Error sending privacy information: {:?}", err),
        _ => ()
    }

    Ok(())
}