use log::{error, warn, info, debug, trace, LevelFilter};
use serenity::{builder, model::{prelude::interaction::application_command::{CommandDataOption, CommandData}, user::User}};
use serenity::model::prelude::command::CommandOptionType;
use serde::{self, Serialize, Deserialize};
use serde_json;

use hyper_tls::HttpsConnector;

use hyper::{Client, Uri};

static URL: &str = "https://api-ratp.pierre-grimaud.fr/v4";

#[derive(Serialize, Deserialize, Debug)]
struct RER {
    line: String,
    slug: String,
    title: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RERs {
    rers: Vec<RER>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StatusRER {
    result: RERs,
}

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("rers")
        .description("Bison futé")
}

pub async fn run(data: &CommandData) -> String {
    // let line = data.options[0].value.as_ref().unwrap().as_str().unwrap();

    // check if line is in array
    // let lines = ["A", "B", "C", "D", "E"];
    // if !lines.contains(&line) {
    //     "Ligne non reconnue".to_string();
    // }

    let uri: Uri = format!("{}/traffic/rers", URL).parse().unwrap();

    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    let res = client
        .get(uri)
        .await;
    
    match res {
        Ok(res) => {

            let status = res.status();
            if !status.is_success() {
                return "Erreur".to_string();
            }

            let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
            let body = String::from_utf8(bytes.to_vec()).unwrap();

            warn!("body: {}", body);

            let status: StatusRER = serde_json::from_slice(&bytes.to_vec()).unwrap();

            let mut message = String::new();

            for (_i, r) in status.result.rers.iter().enumerate() {
                let line = match r.line.as_str() {
                    "A" => ":a:",
                    "B" => ":b:",
                    "C" => ":regional_indicator_c:",
                    "D" => ":regional_indicator_d:",
                    "E" => ":regional_indicator_e:",
                    _ => "X",
                };
                    
                message.push_str(&format!("{} {}\n", line, r.message));
            }

            message
        },
        Err(err) => {
            format!("Error: {}", err)
        }
    }

}
