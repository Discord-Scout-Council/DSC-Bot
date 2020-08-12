/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */
use crate::checks::*;
use crate::prelude::*;

#[command]
#[description = "Adds a badge to a user"]
#[usage("<UserId> <Badge>")]
#[checks(VibeOfficer)]
async fn addbadge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_data = &ctx.data.read().await;
    let pg_pool = bot_data.get::<ConnectionPool>().unwrap();
    let target = match args.current().unwrap().parse::<UserId>() {
        Ok(u) => u,
        Err(e) => return Err(CommandError(e.to_string())),
    };
    args.advance();
    let badge = args.rest();

    sqlx::query!(
        "INSERT INTO badges (userid, badge) VALUES ($1,$2)",
        target.as_u64().to_string(),
        badge
    )
    .execute(pg_pool)
    .await;

    if let Err(e) = msg
        .channel_id
        .send_message(&ctx, |m| {
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
        })
        .await
    {
        return Err(CommandError(e.to_string()));
    }

    Ok(())
}

#[command]
#[description = "Removes a badge from a user"]
#[usage("<User> <Badge>")]
#[checks(VibeOfficer)]
#[min_args(2)]
async fn delbadge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let bot_data = &ctx.data.read().await;
    let pg_pool = bot_data.get::<ConnectionPool>().unwrap();

    {
        let target_user = args.current().unwrap().parse::<UserId>()?;
        args.advance();
        let target_badge = args.rest();

        let num_changed = sqlx::query!(
            "DELETE FROM badges WHERE userid = $1 AND badge = $2",
            target_user.as_u64().to_string(),
            target_badge
        )
        .execute(pg_pool)
        .await
        .map_err(|e| CommandError(e.to_string()))?;

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
}
