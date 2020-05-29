/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use crate::prelude::*;
use serenity::{
    http::GuildPagination,
    model::{
        guild::PartialGuild,
        id::{GuildId, ChannelId},
        invite::{Invite, InviteGuild},
        guild::Guild,
    },
};

#[command]
#[description = "Pings the bot"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.channel_id.say(&ctx.http, "Pong!").await {
        println!("Err sending message: {}", err);
    };

    Ok(())
}

#[command]
#[description = "Provides helpful information about the bot"]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
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
    }).await?;

    Ok(())
}

#[command]
#[description = "Displays information about the server"]
#[only_in(guilds)]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_lock = msg.guild(&ctx.cache).await.unwrap();
    let guild = guild_lock.read().await;

    let member_count = guild.member_count;
    let mut guild_owner = guild.owner_id.to_user(&ctx.http).await.unwrap().name;

    let icon_url = match guild.icon_url() {
        Some(url) => url,
        None => String::from("https://external-content.duckduckgo.com/iu/?u=http%3A%2F%2Fwww.meessendeclercq.be%2Fimages%2Fgallery%2Fartists%2FLDB_Image_Not_Found_web.jpg&f=1&nofb=1")
    };

    guild_owner.push_str("#");
    guild_owner.push_str(
        &guild
            .owner_id
            .to_user(&ctx.http)
            .await
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
    }).await {
        Err(err) => error!("Error sending server count: {:?}", err),
        Ok(_msg) => (),
    }

    Ok(())
}

#[command]
#[description = "Sends a suggestion to the bot developers"]
#[usage("<Suggestion>")]
#[min_args(1)]
async fn botsuggest(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let suggest_channel = ctx.cache.read().await.guild_channel(668964814684422184).unwrap();
    let suggestion = args.rest();
    let guild_arc = &msg.guild(&ctx).await.unwrap();
    let guild = guild_arc.read().await;
    let guild_name = guild.name.clone();
    suggest_channel.read().await.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Bot Suggestion");
            e.description(suggestion);
            e.field("Suggester", &msg.author.name, true);
            e.field("Guild", guild_name, true);

            e
        });

        m
    }).await?;

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Bot Suggestion");
            e.description("Successfully sent your suggestion!");
            e.colour(Colour::DARK_GREEN);

            e
        });

        m
    }).await?;

    Ok(())
}

#[command]
#[description = "Describes what kind of data the bot collects, and what you can do to get your data removed."]
async fn privacy(ctx: &Context, msg: &Message) -> CommandResult {
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
    }).await {
        Err(err) => error!("Error sending privacy information: {:?}", err),
        _ => ()
    }

    Ok(())
}

/*
#[command]
#[description = "Returns a list of member servers, with invites if available. This command may take a while to process."]
async fn servers(ctx: &Context, msg: &Message) -> CommandResult {
    let http_cache = &ctx.http;
    let current_info = http_cache.get_current_application_info().await?;
    let bot_id = current_info.id.as_u64();
    http_cache.broadcast_typing(*msg.channel_id.as_u64());
    let guild_info_list = match http_cache.get_guilds(&GuildPagination::After(GuildId(0)), 50).await {
        Ok(v) => v,
        Err(err) => return Err(CommandError(err.to_string())),
    };

    let mut guild_ids: Vec<GuildId> = Vec::new();
    for info in guild_info_list.iter() {
        guild_ids.push(info.id);
    }

    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for guild in guild_info_list.iter() {
        if *guild.id.as_u64() == 363354951071694848u64
            || *guild.id.as_u64() == 646540745443901469u64
        {
            continue;
        }
        let guild_name = &guild.name;
        let partial_guild = guild.id.to_partial_guild(http_cache).await?;
        let guild_owner = match partial_guild.owner_id.to_user(http_cache).await {
            Ok(u) => u,
            Err(err) => return Err(CommandError(err.to_string())),
        };
        let owner_name = format!("<@{}>", guild_owner.id.as_u64().to_string());
        let guild_arc = match partial_guild.id.to_guild_cached(&ctx).await {
            Some(a) => a,
            None => return Err(CommandError("Could not open the guild Arc.".to_string())),
        };
        let guild = guild_arc.read();
        let default_channel_arc = match guild.default_channel(UserId(*bot_id)) {
            Some(c) => c,
            None => return Err(CommandError("Could not find default channel.".to_string())),
        };
        let default_channel = default_channel_arc.read();
        let invite: String = match Invite::create(http_cache, default_channel.id, |mut i| {
            i.max_age(0);
            i
        }).await {
            Ok(i) => i.url(),
            Err(err) => {
                warn!("Could not create invite for {}", guild_name);
                "No invite provided".to_string()
            }
        };
        fields.push((
            guild_name.clone(),
            format!("{}\n{}", owner_name, invite),
            false,
        ));
    }

    if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("DSC Server List");
            e.description("This is a list of all DSC Member Servers");
            e.fields(fields);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }).await {
        return Err(CommandError(err.to_string()));
    }

    Ok(())
}
*/

#[command]
#[description = "Sends a server to be nominated for membership in DSC."]
#[min_args(2)]
pub async fn nominate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let invite_url = args.single::<String>()?;
    let invite_code = match invite_url.split("/").last() {
        Some(url) => url,
        None => {
            return Err(CommandError(format!("Error fetching invite code")));
        }
    };
    let invite: Invite = Invite::get(&ctx, invite_code, true).await?;
    let demographic = args.single::<String>()?;

    let target_server: InviteGuild = match invite.guild {
        Some(guild) => guild,
        None => {
            return Err(CommandError(format!("Could not fetch nominee guild")));
        }
    };

    let target_name = target_server.name;
    let member_count = match invite.approximate_member_count {
        Some(count) => count,
        None => 0u64,
    };


    let notify_channel = ChannelId(668964814684422184);
    if let Err(err) = notify_channel.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("New nominee");
            e.fields(vec![
                ("Demographic", demographic, true),
                ("Member Count", member_count.to_string(), true),
                ("Invite Link", invite_url, true),
            ]);
            e.description(target_name);
            e
        });
        m
    }).await {
        return Err(CommandError(format!("Could not send nominee: {:?}", err)));
    }

    if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Nomination Sent");
            e.color(Colour::DARK_GREEN);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }).await {
        return Err(CommandError(err.to_string()));
    }


    Ok(())
}

#[command]
#[description = "Starts a vote in the specified channel"]
#[usage("<channel> <Vote>")]
#[min_args(2)]
pub async fn startvote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let target_channel = args.single::<ChannelId>()?;
    let vote_thing = args.rest();
    crate::util::run_yesno_vote(&ctx, target_channel, vote_thing.to_string()).await?;

    Ok(())
}