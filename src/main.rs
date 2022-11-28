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

use local_env::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ratio" => commands::ratio::run(&command.data),
                "rer" => commands::rer::run(&command.data).await,
                "rers" => commands::rers::run(&command.data).await,
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
                .create_application_command(|command| commands::rer::register(command))
                .create_application_command(|command| commands::rers::register(command))
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

    let level: LevelFilter = match env::var("RUST_LOG") {
        Ok(val) => match val.as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        },
        Err(_) => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
        .filter(None, level)
        .target(env_logger::Target::Stdout)
        .write_style(env_logger::fmt::WriteStyle::Always)
        .init();

    local_env::check_vars();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(DISCORD_TOKEN.as_str(), intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start_shards(*SHARD_NB).await {
        error!("Client error: {:?}", why);
    }
}