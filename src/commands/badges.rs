/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */
use crate::checks::*;
use crate::prelude::*;
use rusqlite::params;

#[command]
#[description = "Adds a badge to a user"]
#[usage("<UserId> <Badge>")]
#[checks(VibeOfficer)]
async fn addbadge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let badge_db = get_badge_db();
    let target = match args.current().unwrap().parse::<UserId>() {
        Ok(u) => u,
        Err(e) => return Err(CommandError(e.to_string())),
    };
    args.advance();
    let badge = args.rest();
    {
    let mut stmt = match badge_db.prepare("INSERT INTO badges (userid, badge) VALUES (?1,?2)") {
        Ok(s) => s,
        Err(e) => return Err(CommandError(e.to_string())),
    };


    match stmt.execute(params![target.as_u64().to_string(), badge]) {
        Ok(_u) => {
            
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };
}
    if let Err(e) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Badge Subsystem");
            e.description("Successfully added badge.");
            e.field("Badge", badge, true);
            e.colour(Colour::DARK_GREEN);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }).await {
        return Err(CommandError(e.to_string()));
    }

    Ok(())
}


#[command]
#[description = "Removes a badge from a user"]
#[usage("<User> <Badge>")]
#[checks(VibeOfficer)]
async fn delbadge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let badge_db = get_badge_db();
    let mut num_changed: usize;
    {
    let target_user = args.current().unwrap().parse::<UserId>()?;
    args.advance();
    let target_badge = args.rest();
    
    let mut badge_stmt = badge_db.prepare("DELETE FROM badges WHERE userid = ?1 AND badge = ?2")?;
    num_changed = badge_stmt.execute(params![target_user.as_u64().to_string(), target_badge])?;
    }
    match num_changed {
        0 => {
            if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Badge Subsystem");
                    e.description("Error removing badge. Either you provided an invalid badge, or the user does not have the badge.");
                    e.colour(Colour::RED);
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
        },
        _ => {
            if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("Badge Subsystem");
                    e.description("Successfully removed badge");
                    e.colour(Colour::DARK_GREEN);
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
        },
    }

    Ok(())
}