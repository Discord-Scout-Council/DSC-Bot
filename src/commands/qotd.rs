/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */

use serenity::framework::standard::{macros::command, Args, CommandResult, StandardFramework};
use serenity::model::id::UserId;
use serenity::{model::channel::Message, model::guild::Member, prelude::*};
use crate::checks::*;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use std::cmp::Ordering;
use crate::util::data::get_pickle_database;

#[derive(Eq)]
struct Question {
    num: i32,
    text: String
}

impl Ord for Question {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num.cmp(&other.num)
    }
}

impl PartialOrd for Question {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Question {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

#[command]
#[description = "Manages the Question of the Day"]
#[sub_commands(add, run)]
pub fn qotd(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Base command");

    Ok(())

}

#[command]
#[description = "Adds a question of the day to the rotation"]
#[checks(Moderator)]
pub fn add(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut message = String::from("Added ");
    message.push_str(&args.rest());
    message.push_str(&" to the question of the day");
    println!("Opening QOTDatabase");
    let mut db = PickleDb::load_yaml("qotd.db", PickleDbDumpPolicy::AutoDump)?;
    println!("Getting all keys");
    let mut db_keys = db.get_all();

    let mut questions: Vec<Question> = Vec::new();

    println!("Sorting questionsn");
    for q in db_keys.iter() {
        questions.push(Question { num: q.parse::<i32>().unwrap(), text: db.get(q).unwrap()});
    }

    questions.sort();
    let highest_num = match questions.first() {
        Some(q) => q.num,
        None => 0,
    };
    let current_num = highest_num + 1;
    //* This code should go in the `run` command.
    //let new_question: String = db.get(&current_num.to_string()).unwrap();
    // let previous_num: i32 = guild_cache.get(&"previous_qotd").unwrap();
    println!("Adding question");
    db.set(&current_num.to_string(), &args.rest());
    msg.channel_id.say(&ctx.http, message).unwrap();
    Ok(())
}

#[command]
#[description = "Runs the Question of the Day based on the last run question"]
#[checks(Moderator)]
pub fn run(ctx: &mut Context, msg: &Message) -> CommandResult {
    let db = get_pickle_database(msg.guild_id.unwrap().as_u64(), &String::from("qotd.db"));
    let mut guild_cache = get_pickle_database(msg.guild_id.unwrap().as_u64(), &"cache.db");
    let current_num: i32 = guild_cache.get::<i32>("current_qotd").unwrap() + 1;
    let next_question = Question {num: current_num + 1, text: db.get(&current_num.to_string()).unwrap()};

    println!("Finding qotd channel");
    let guild_arc = msg.guild(&ctx).unwrap();
    let guild = guild_arc.read();
    let qotd_channel = guild.channel_id_from_name(&ctx, "qotd-questions").unwrap();

    qotd_channel.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Question of the Day");
            e.description(&next_question.text);
            e.footer(|f| {
                let mut text: String = String::from("Requested by ");
                text.push_str(&msg.author.name);
                f.text(text);

                f
            });

            e
        });

        m
    })?;
    println!("Sent message");

    guild_cache.set("current_qotd", &current_num)?;

    Ok(())
}