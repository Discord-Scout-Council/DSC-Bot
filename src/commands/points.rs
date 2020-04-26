/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use pickledb::*;
use serenity::framework::standard::{macros::command, CommandResult, StandardFramework};
use serenity::{
    model::{channel::Message, id::UserId},
    prelude::*,
};

use crate::util::data::get_pickle_database;

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

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(&guild.name);

                e.fields(vec![
                    ("1.", &leaderboard[0], false),
                    ("2.", &leaderboard[1], false),
                    ("3.", &leaderboard[2], false),
                    ("4.", &leaderboard[3], false),
                    ("5.", &leaderboard[4], false),
                ]);

                e
            });

            m
        })
        .unwrap();

    Ok(())
}
