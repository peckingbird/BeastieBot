use std::env;
use std::fs;
use std::collections::HashMap;
use dotenv::dotenv;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;

struct Handler;

#[derive(Serialize, Deserialize, Debug)]
struct Beastie {
    name: String,
    #[serde(rename(deserialize = "ba"))]
    body_pow: u32,
    #[serde(rename(deserialize = "bd"))]
    body_def: u32,
    #[serde(rename(deserialize = "ha"))]
    spirit_pow: u32,
    #[serde(rename(deserialize = "hd"))]
    spirit_def: u32,
    #[serde(rename(deserialize = "ma"))]
    mind_pow: u32,
    #[serde(rename(deserialize = "md"))]
    mind_def: u32,
    #[serde(rename(deserialize = "desc"))]
    description: String
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("!beastie") {
            let bot_response = handle_beastie_command(&msg.content);
            let _ = msg.channel_id.say(&ctx.http, bot_response).await;
        }
    }
}

fn handle_beastie_command(msg_content: &str) -> String {
    if msg_content.len() < 10 {
        return String::from("You must say the name of the beastie...");
    }
    
    let beastie_name = normalise_beastie_name(&msg_content[9..]);
    let beastie = get_beastie(&beastie_name);
    match beastie {
        Some(b) => to_detail_string(&b),
        None => String::from("Sorry, I couldn't find that beastie")
    }
}

fn to_beastie(value: Value) -> Beastie {
    serde_json::from_value(value).unwrap()
}

fn to_detail_string(beastie: &Beastie) -> String {
    let mut details = format!("__**{}:**__\n", beastie.name);
    details.push_str("```ansi\n");
    details.push_str(&format!("[2;33mBody[0m    POW/DEF: {}/{}\n",
                              pad_stat(beastie.body_pow),
                              pad_stat(beastie.body_def)));
    details.push_str(&format!("[2;31mSpirit[0m  POW/DEF: {}/{}\n",
                              pad_stat(beastie.spirit_pow),
                              pad_stat(beastie.spirit_def)));
    details.push_str(&format!("[2;34mMind[0m    POW/DEF: {}/{}",
                              pad_stat(beastie.mind_pow),
                              pad_stat(beastie.mind_def)));
    details.push_str("```\n");
    details.push_str(&format!("> {}", beastie.description));
    details
}

fn get_beastie(beastie_name: &str) -> Option<Beastie> {
    let project_root = env::var("CARGO_MANIFEST_DIR")
        .expect("Cargo directory should be set");
    let beastie_file_path = project_root + &String::from("/src/beastie_data.json");
    let beastie_file = fs::File::open(beastie_file_path)
        .expect("Beastie_data should be opened as read-only");
    let beastie_map: HashMap<String, Value> = serde_json::from_reader(beastie_file)
        .expect("JSON should be a valid map");
    beastie_map.iter()
        .map(|(_k, v)| to_beastie(v.clone()))
        .find(|beastie| beastie.name == beastie_name)
}

fn pad_stat(stat: u32) -> String {
    match stat {
        s if s < 10 => "[2;30m00[0m".to_owned() + &stat.to_string(),
        s if s < 100 => "[2;30m0[0m".to_owned() + &stat.to_string(),
        _ => stat.to_string()
    }
}

fn normalise_beastie_name(input: &str) -> String {
    let first_letter = input[0..1].to_uppercase();
    let rest = &input[1..].to_lowercase();
    first_letter + &rest
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

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_beastie_command_gets_details_for_beastie() {
        let input = "!beastie trat";
        let expected_output = "__**Trat:**__
```ansi
[2;33mBody[0m    POW/DEF: [2;30m0[0m81/[2;30m0[0m49
[2;31mSpirit[0m  POW/DEF: [2;30m0[0m23/[2;30m0[0m93
[2;34mMind[0m    POW/DEF: [2;30m0[0m64/[2;30m0[0m84```
> They are often found in dumpsters. When they outgrow their can, Trats will gather with many others to exchange for larger cans.";

        let actual_output = handle_beastie_command(input);

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn handle_beastie_command_copes_when_no_beastie_is_provided() {
        let input = "!beastie ";
        let expected_output = "You must say the name of the beastie...";

        let actual_output = handle_beastie_command(&input);

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn handle_beastie_command_copes_when_beastie_is_not_found() {
        let input = "!beastie sfjalf";
        let expected_output = "Sorry, I couldn't find that beastie";

        let actual_output = handle_beastie_command(&input);

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn to_detail_string_formats_beastie_details() {
        let input: Beastie = Beastie {
            name: String::from("Test Beastie"),
            body_pow: 1,
            body_def: 2,
            spirit_pow: 3,
            spirit_def: 4,
            mind_pow: 5,
            mind_def: 6,
            description: String::from("Test Description")
        };

        let expected_output = "__**Test Beastie:**__
```ansi
[2;33mBody[0m    POW/DEF: [2;30m00[0m1/[2;30m00[0m2
[2;31mSpirit[0m  POW/DEF: [2;30m00[0m3/[2;30m00[0m4
[2;34mMind[0m    POW/DEF: [2;30m00[0m5/[2;30m00[0m6```
> Test Description";

        let actual_output = to_detail_string(&input);

        assert_eq!(actual_output, expected_output);
    }
    
    #[test]
    fn pad_stat_double_pads_stats_lower_than_10() {
        let input = 9;
        let expected_output = "[2;30m00[0m9";

        let actual_output = pad_stat(input);

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn pad_stat_pads_stats_lower_than_100() {
        let input = 99;
        let expected_output = "[2;30m0[0m99";

        let actual_output = pad_stat(input);

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn pad_stat_does_not_pad_stats_at_or_over_100() {
        let input = 100;
        let expected_output = "100";

        let actual_output = pad_stat(input);

        assert_eq!(actual_output, expected_output);
    }
    
    #[test]
    fn normalize_beastie_name_handles_all_upper() {
        let input = "TRAT";
        let expected_output = "Trat";

        let actual_output = normalise_beastie_name(input);

        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn normalize_beastie_name_handles_all_lower() {
        let input = "trat";
        let expected_output = "Trat";

        let actual_output = normalise_beastie_name(input);

        assert_eq!(actual_output, expected_output);        
    }

    #[test]
    fn normalize_beastie_name_handles_a_mixture_of_cases() {
        let input = "tRaT";
        let expected_output = "Trat";

        let actual_output = normalise_beastie_name(input);

        assert_eq!(actual_output, expected_output);        
    }    
}
