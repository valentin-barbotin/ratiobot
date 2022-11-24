use std::env;

use serenity::{async_trait, model::{prelude::Message, error}};
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use std::io::{Write, Result};
use log::{error, warn, info, debug, trace, LevelFilter};
use env_logger;
use dotenv::dotenv;

mod commands;
mod local_env;

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
                error!("Error responding to command: {:?}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!boulot" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Boulot pierre").await {
                error!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::ratio::register(command))
        })
        .await;
        if commands.is_err() {
            error!("Error registering slash commands: {:?}", commands);
        }

        let res = Command::create_global_application_command(&ctx.http, |command| {
            commands::ratio::register(command)
        })
        .await;

        match res {
            Ok(_) => info!("Successfully registered slash commands"),
            Err(why) => error!("Error registering slash commands: {:?}", why),
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .write_style(env_logger::fmt::WriteStyle::Always)
        .init();

    local_env::check_vars();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start_shards(env::var("SHARD_NB").unwrap().parse().unwrap()).await {
        error!("Client error: {:?}", why);
    }
}