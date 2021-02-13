use futures::future::join_all;
use riven::consts::Region;
use riven::{
    models::match_v4::{Match, MatchReference},
    RiotApi,
};
use serde::Serialize;
use std::env;

#[macro_use]
extern crate rocket;

use rocket_contrib::json::Json;

/// main todos:
/// Rocket API - input summoner name => get time series back of playtimes per day
/// Sled database - add summoner name to list for scraping to keep up-to-date database of match
/// history
/// netlify frontend
/// everything is NA now, need to parameterize
/// remove all the expect/unwrap
/// todo test known time/date matches

const MAX_RETRIES: usize = 20;

/// uPlot expects a list of timestamps and a list of data points
#[derive(Serialize)]
struct MatchInfo {
    times: Vec<i64>,
    values: Vec<i64>,
}

#[get("/<summoner>")]
async fn matches(summoner: String) -> Json<MatchInfo> {
    let matches = get_match_history(&summoner)
        .await
        .expect("get match history");
    let match_futures = matches
        .iter()
        .map(|m| get_match(m.game_id))
        .collect::<Vec<_>>();
    let matches = join_all(match_futures).await;
    let (mut times, mut values): (Vec<_>, Vec<_>) = matches
        .iter()
        .map(|m| m.as_ref().expect("get match ok"))
        .map(|m| (m.game_creation / 1000, m.game_duration / 60))
        .unzip();
    times.reverse();
    values.reverse();
    Json(MatchInfo { times, values })
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/matches", routes![matches])
}

async fn get_match_history(
    summoner: &str,
) -> Result<Vec<MatchReference>, Box<dyn std::error::Error>> {
    let api_key = env::var("RIOTAPIKEY").expect("RIOTAPIKEY environment variable set.");
    let riot_api = RiotApi::with_key(api_key); // TODO once_cell

    let name = summoner;
    let summoner = riot_api
        .summoner_v4()
        .get_by_summoner_name(Region::NA, summoner)
        .await
        .expect("Get summoner failed.")
        .expect("There is no summoner with that name.");

    // println!("\n{}\n", &summoner.account_id);

    let mut start_index = 0;
    let mut matches = vec![];
    loop {
        let mut match_list = riot_api
            .match_v4()
            .get_matchlist(
                Region::NA,
                &summoner.account_id,
                None,              // Some(start),
                Some(start_index), // Some(0),
                None,
                None, // Some(start + 604800000),
                None,
                None,
                None,
            )
            .await
            .expect("Get matchlist failed.")
            .expect("No matchlist for account id.");
        if match_list.matches.is_empty() {
            break;
        }
        matches.append(&mut match_list.matches);
        start_index += 100;
    }

    println!("found {} matches for {}", matches.len(), name);
    Ok(matches)
}

async fn get_match(game_id: i64) -> Result<Match, &'static str> {
    let api_key = env::var("RIOTAPIKEY").expect("RIOTAPIKEY environment variable set.");
    let riot_api = RiotApi::with_key(api_key); // TODO once_cell

    // do x retries max
    for _ in 0..MAX_RETRIES {
        let match_ = riot_api.match_v4().get_match(Region::NA, game_id).await;

        if let Ok(ok_match) = match_ {
            if let Some(m) = ok_match {
                return Ok(m);
            }
        }
    }

    Err("Failed to get match")
}
