/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use pickledb::*;
use crate::prelude::*;
use crate::checks::*;

use crate::util::data::get_pickle_database;

use log::{error, debug};

use std::cmp::Ordering;

#[derive(Eq)]
struct UserPoints {
    id: u64,
    points: i32,
}

struct UserIdPoints {
    id: UserId,
    points: i32,
}

impl Ord for UserPoints {
    fn cmp(&self, other: &Self) -> Ordering {
        other.points.cmp(&self.points)
    }
}

impl PartialOrd for UserPoints {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for UserPoints {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points
    }
}

#[command]
#[description = "See how many points you have earned by chatting"]
#[only_in(guilds)]
pub fn points(ctx: &mut Context, msg: &Message) -> CommandResult {
    let db = get_pickle_database(msg.guild_id.unwrap().as_u64(), &"points.db");
    let author_id = msg.author.id.as_u64();

    let points = match db.get(&author_id.to_string()) {
        Some(points) => points,
        None => 0,
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            let mut title = msg.author.name.clone();
            title.push_str("'s profile");

            e.title(title);
            e.field("Points:", points.to_string(), false);

            e
        });

        m
    });

    Ok(())
}

#[command]
#[description = "Displays the top ten users on the leaderboard"]
#[only_in(guilds)]
pub fn leaderboard(ctx: &mut Context, msg: &Message) -> CommandResult {
    let db = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "points.db");
    let mut db_keys = db.get_all();

    let mut points: Vec<UserPoints> = Vec::new();

    for s in db_keys.iter() {
        points.push(UserPoints {
            id: s.parse::<u64>().unwrap(),
            points: db.get(s).unwrap(),
        });
    }

    let mut points_vec: Vec<&UserPoints> = points.iter().collect();
    points_vec.sort();

    let mut points_id_vec: Vec<UserIdPoints> = Vec::with_capacity(points_vec.len());

    for p in points_vec {
        let user_id: UserId = p.id.into();
        points_id_vec.push(UserIdPoints {
            id: user_id,
            points: p.points,
        });
    }

    let mut topList: Vec<&UserIdPoints> = vec![];

    for (i, p) in points_id_vec.iter().enumerate() {
        if (i <= 5) {
            topList.push(&p);
        } else {
            break;
        }
    }

    let guild_arc = msg.guild(&ctx).unwrap();
    let guild = guild_arc.read();

    let mut leaderboard: Vec<String> = Vec::with_capacity(topList.len());

    for p in &topList {
        let mut message = p.id.to_user(&ctx).unwrap().name;
        message.push_str(" - ");
        message.push_str(&p.points.to_string());
        leaderboard.push(message);
    }

    let mut fields: Vec<(String, String, bool)> = Vec::with_capacity(topList.len());
    for (i, p) in topList.iter().enumerate() {
        let num = i + 1;
        fields.push((
            num.to_string(),
            format!(
                "{} - {}",
                p.id.to_user(&ctx).unwrap().name,
                &p.points.to_string()
            ),
            false,
        ));
    }

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(&guild.name);

                e.fields(fields);

                e
            });

            m
        })
        .unwrap();

    Ok(())
}

#[command]
#[description="Allows a moderator to modify a user's points for the server"]
#[num_args(2)]
#[only_in(guilds)]
#[checks(Moderator)]
pub fn modpoints(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut points_db = get_pickle_database(&msg.guild_id.unwrap().as_u64(), "points.db");
    let user: &UserId = &args.current().unwrap().parse::<UserId>().unwrap();
    debug!("Attempting to locate guild via cache");
    let guild_arc = &msg.guild(&ctx).unwrap();
    let guild = guild_arc.read();
    args.advance();
    let points_mod = &args.current().unwrap().parse::<i64>().unwrap();
    let current_points = match points_db.get::<u64>(&user.as_u64().to_string()) {
        Some(i) => i,
        None => {
            error!("Could not find {}'s points for {}", user.to_user(&ctx).unwrap().name, &guild.name);
            0
        }
    };
    if current_points as i64 + points_mod < 0 {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Points");
                e.description("Cannot set a user's points to less than 0.");
                e.colour(Colour::RED);
                e
            });
            m
        })?;
    } else {
        let new_points = current_points as i64 + points_mod;
        points_db.set(&user.as_u64().to_string(), &new_points);
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Points");
                e.description("Successfully modified the user's points!");
                e.fields(vec![
                    ("User", &user.to_user(&ctx).unwrap().name, true),
                    ("Old Points", &current_points.to_string(), true),
                    ("New Points", &new_points.to_string(), true),
                ]);
                e
            });
            m
        })?;
    }


    Ok(())
}