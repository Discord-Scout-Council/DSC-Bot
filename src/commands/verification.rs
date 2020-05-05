/*
 *   Copyright (c) 2020 Owen Salter <owen@devosmium.xyz>
 *   All rights reserved.
 */
use crate::prelude::*;

#[command]
#[description = "Allows users to set their age group as over/under 18"]
#[usage("<over/under>")]
#[num_args(1)]
pub fn age(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut age_db = get_global_pickle_database("age.db");
    let overunder = args.current().unwrap();
    if overunder == "over" {
        match age_db.set(&msg.author.id.as_u64().to_string(), &"Over") {
            Err(err) => {
                error!("Error setting {}'s age: {:?}", &msg.author.name, err);
                return Err(CommandError(err.to_string()));
            }
            _ => (),
        }
    } else if overunder == "under" {
        match age_db.set(&msg.author.id.as_u64().to_string(), &"Under") {
            Err(err) => {
                error!("Error setting {}'s age: {:?}", &msg.author.name, err);
                return Err(CommandError(err.to_string()));
            }
            _ => (),
        }
    } else {
        match msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Age Verification");
                e.description("You provided an invalid age group. Please choose either <over> 18 or <under> 18 and run the command again.");
                e.color(Colour::RED);
                e.footer(|f| {
                    f.text("DSC Bot | Powered by Rusty Development");
                    f
                });
                e
            });
            m
        }) {
            Err(err) => {
                error!("Error sending error message response in channel {}: {:?}", &msg.channel_id.as_u64().to_string(), err);
                return Err(CommandError(err.to_string()));
            },
            _ =>  {
                return Err(CommandError(String::from("Invalid Arguments")));
            },
        }
    }

    match msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Age Verification");
            e.description(format!(
                "Successfully set your age group as {} 18",
                overunder.to_lowercase()
            ));
            e.color(Colour::DARK_GREEN);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e
        });
        m
    }) {
        Err(err) => {
            error!(
                "Error sending age success response in channel {}: {:?}",
                &msg.channel_id.as_u64().to_string(),
                err
            );
            return Err(CommandError(err.to_string()));
        }
        _ => (),
    }

    info!("Set age for {} to {} 18", &msg.author.name, overunder);

    Ok(())
}

#[command]
#[description = "Gives instructions on how to verify your Awards and Advancements"]
pub fn verify(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(err) = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Verification");
            e.description("In order to verify your roles, please send a direct message to the bot, and attach an image of your proof.\n\nScoutbook screenshots that clearly show a completed award are valid proof for BSA awards.");
            e.fields(vec![
                ("Eagle Scout", "Patch, Card, or Certificate", true),
                ("Summit/Silver", "Patch, Card, or Certificate", true),
                ("Camp Staff", "Name tag or shirt", true),
                ("YPT", "Certificate. PDF is acceptable for this verification", true),
                ("OA Honor", "Sash or membership card", true),
                ("Quartermaster", "Medal, Patch, Card, and Certificate", true)
            ]);
            e.footer(|f| {
                f.text("DSC Bot | Powered by Rusty Development");
                f
            });
            e.colour(Colour::BLUE);

            e
        });
        m
    }) {
        error!("Error sending verify instructions: {:?}", err);
        return Err(CommandError(err.to_string()));
    }

    Ok(())
}
