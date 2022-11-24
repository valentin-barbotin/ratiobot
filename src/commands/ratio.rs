use serenity::{builder, model::{prelude::interaction::application_command::{CommandDataOption, CommandData}, user::User}};
use serenity::model::prelude::command::CommandOptionType;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("ratio")
        .description("Ratio")
        .create_option(|option| {
            option
                .name("user")
                .description("the user")
                .kind(CommandOptionType::User)
                .required(true)
        })
}

pub fn run(data: &CommandData) -> String {

    let list = data.resolved.users.values().collect::<Vec<&User>>();
    let user = list[0]; // only one user in the list
    
    format!("Ratio {}", user.name)
    // "https://media.discordapp.net/attachments/969210376862457946/1045287592003903488/352617755955953664.jpg?width=668&height=605".to_string()
}
