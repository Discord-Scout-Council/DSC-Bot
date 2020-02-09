use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
    framework::standard::{StandardFramework, macros::group}
};
mod commands;
mod util;
use crate::commands::*;

#[group]
#[commands(ping, kick)]
struct General;

struct Handler;
impl EventHandler for Handler {

    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} logged in successfully!", ready.user.name);
    }
}

fn main() {
    let config: util::BotConfig = util::parse_config();
    
    let token = config.token.clone();

    

    let mut client = Client::new(token, Handler).expect("Err creating client");

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(&config.prefix.to_string())).group(&GENERAL_GROUP));

    if let Err(err) = client.start() {
        eprintln!("{:?}", err);
    }
}

fn parse_command(msg: &Message) -> Result<String, String> {
    let command_list = commands::define_commands();
    let split = msg.content.split_whitespace().collect::<Vec<&str>>();
    let command = split[0].to_string();

    let mut command_key = String::new();
    let mut matched = false;

    for c in command_list.iter() {
        if c.key == command {
            println!("Matched to {}", c.key);
            matched = true;
            command_key = c.key.clone();
        } 
    }
    if matched {
        Ok(command_key)
    } else {
        Err(String::from("Could not match command"))
    }
}
