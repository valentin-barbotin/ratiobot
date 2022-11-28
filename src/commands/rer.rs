use log::{error, warn, info, debug, trace, LevelFilter};
use serenity::{builder, model::{prelude::interaction::application_command::{CommandDataOption, CommandData}, user::User}, http::request::RequestBuilder};
use serenity::model::prelude::command::CommandOptionType;
use serde::{self, Serialize, Deserialize};
use serde_json;
use url_params_serializer::to_url_params;

use hyper_tls::HttpsConnector;

use hyper::{Client, Uri, Request};

use crate::local_env::TWITTER_TOKEN;

use super::*;

static URL: &str = "https://api.twitter.com";

macro_rules! params_vec_to_string {
    ($vec:expr) => {
        $vec.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("&")
    };
}


#[derive(Deserialize)]
struct Tweet {
    edit_history_tweet_ids: Vec<String>,
    id: String,
    text: String,
}

#[derive(Deserialize)]
struct Tweets {
    data: Vec<Tweet>,
}

#[derive(Serialize)]
struct Params {
    query: String,
    max_results: u8,
    sort_order: String,
}

#[derive(PartialEq, Debug)]
enum RERState {
    OK,
    Warning,
    Default,
}

static OK_LIST: [&str; 3] = ["Fin de stationnement", "Le train repart", "le trafic est rétabli"];
static WARNING_LIST: [&str; 5] = ["Le trafic est perturbé", "stationne en raison", "le trafic est rétabli", "retard", "gêne de circulation"];

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("rer")
        .description("Bison futé")
        .create_option(|option| {
            option
                .name("rer")
                .description("Ligne de RER")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

fn search_indicator(text: &str) -> RERState {
    for w in OK_LIST.iter() {
        if text.contains(w) {
            debug!("ok: {}", w);
            return RERState::OK
        }
    }

    for w in WARNING_LIST.iter() {
        if text.contains(w) {
            debug!("warn: {}", w);
            return RERState::Warning
        }
    }

    RERState::Default
}

fn get_line(line: &str) -> String {
    let res = match line {
        "A" => "RER_A",
        "B" => "RERB",
        "C" => "RERC_SNCF",
        "D" => "RERD_SNCF",
        "E" => "RERE_T4_SNCF",
        _ => "RER_A", // default
    };

    String::from(res)
}

pub async fn run(data: &CommandData) -> String {
    let line = data.options[0].value.as_ref().unwrap().as_str().unwrap();

    let line = line.to_uppercase();
    let line = line.as_str();

    // check if line is in array
    let lines = ["A", "B", "C", "D", "E"];
    if !lines.contains(&line) {
        "Ligne non reconnue".to_string();
    }

    let line = get_line(line);

    let params = Params {
        query: format!("from%3A{}", line),
        max_results: 30,
        sort_order: "relevancy".to_string(),
    };
    let url = to_url_params(params);
    let params = params_vec_to_string!(url);
    let uri: Uri = format!("{}/2/tweets/search/recent?{}", URL, params).parse().unwrap();
    debug!("uri: {}", uri);

    let https = HttpsConnector::new();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);
    
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", TWITTER_TOKEN.as_str()))
        .body(hyper::Body::empty());

    match req {
        Ok(req) => {
            let res = client.request(req).await;
            match res {
                Ok(res) => {
                    let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
                    
                    let tweets: Tweets = match serde_json::from_slice(&bytes.to_vec()) {
                        Ok(tweets) => tweets,
                        Err(e) => {
                            warn!("error: {}", e);
                            return "Impossible de récuperer les informations requises".to_string();
                        }
                    };


                    let mut current_state = RERState::Default;
                    let mut msg: Option<String> = None;
                    // relevancy (oldest first)
                    for tweet in tweets.data {
                        let state = search_indicator(&tweet.text);
                        if state != RERState::Default {
                            debug!("state: {:?}, {}", state, tweet.text);
                            current_state = state;
                            msg = Some(tweet.text);
                        }
                    }

                    let final_state =  match current_state {
                        RERState::OK => "Tout va bien".to_string(),
                        RERState::Warning => "Il y'a un problème".to_string(),
                        RERState::Default => "Je ne suis pas sûr mais le trafic à l'air normal".to_string(),
                    };
                    
                    let res = match msg {
                        Some(m) => format!("Ligne {}: {}\n\nTweet:\n{}", line, final_state, m),
                        None => format!("Ligne {}: {}", line, final_state),
                    };

                    return res;
                },
                Err(e) => {
                    error!("Error: {}", e);
                }
            }

        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    "Impossible de determiner l'etat de la ligne".to_string()

}



#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn parse_params() {
        let line = "A";

        let line = get_line(line);

        let params = Params {
            query: format!("from%3A{}", line),
            max_results: 30,
            sort_order: "relevancy".to_string(),
        };
        let url: Vec<(String, String)> = to_url_params(params);
        // slice to string
        let queryparams = params_vec_to_string!(url);

        // println!("{}", url);
        assert_eq!(queryparams, "query=from:RER_A&max_results=10");
    }
}