use std::env;

use serenity::{async_trait, model::prelude::Message};
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use dotenv::dotenv;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ratio" => commands::ratio::run(&command.data),
                _ => "Not implemented".to_string(),
            };

            if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!boulot" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Boulot pierre").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);


        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::ratio::register(command))
        })
        .await;
        if commands.is_err() {
            println!("Error registering slash commands: {:?}", commands);
        }

        let res = Command::create_global_application_command(&ctx.http, |command| {
            commands::ratio::register(command)
        })
        .await;

        match res {
            Ok(_) => println!("Registered commands"),
            Err(why) => println!("Error registering commands: {:?}", why),
        }
    }
}

#[tokio::main]
async fn main() {

    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start_shards(4).await {
        println!("Client error: {:?}", why);
    }
}