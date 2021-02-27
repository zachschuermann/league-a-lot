use serde::Serialize;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use rocket::State;
use std::collections::VecDeque;

use league_a_lot::{get_all, get_match_history, get_match_history_since};

#[macro_use]
extern crate rocket;

use rocket_contrib::json::Json;

const RECENT_LEN: usize = 10;

/// main todos:
/// Rocket API - input summoner name => get time series back of playtimes per day
/// Sled database - add summoner name to list for scraping to keep up-to-date database of match
/// history
/// netlify frontend
/// everything is NA now, need to parameterize
/// remove all the expect/unwrap
/// todo test known time/date matches
///
/// rocket doesn't manage any state in memory, this makes it durable, but slower since we always
/// query sled.
///
/// check out sled examples/structured.rs

// state
struct Recent(Arc<Mutex<VecDeque<String>>>);
// NB Rocket can't handle state of same type in multiple instances, so we create a new type for
// each sled database.
struct Index(sled::Db);
struct Db(sled::Db);

/// uPlot expects a list of timestamps and a list of data points
#[derive(Serialize)]
struct MatchInfo {
    ok: bool,
    times: Vec<i64>,
    values: Vec<i64>,
}

/// take username for scraping
/// 1. check if exists in db index
/// yes.
///     2. scrape from last saved time to now - save in db
/// no.
///     2. scrape from now till old as we can - save in db
/// 3. update db index with
///    - name
///    - last scraped timestamp
#[get("/matches/<summoner>")]
async fn matches(recent: State<'_, Recent>, index: State<'_, Index>, db: State<'_, Db>, summoner: String) -> Json<MatchInfo> {
    let new_matches = match index.0
        .get(summoner.as_bytes())
        .expect("get summoner from index")
    {
        Some(time) => get_match_history_since(&summoner, ivec_to_i64(time)).await,
        None => get_match_history(&summoner).await,
    };

    let matches = match new_matches {
        Ok(ms) => ms,
        Err(_) => {
            println!("ERROR");
            return Json(MatchInfo {
                ok: false,
                times: vec![],
                values: vec![],
            })
        }
    };

    let (times, values) = get_all(matches).await;

    let summoner_tree = db.0
        .open_tree(summoner.as_bytes())
        .expect("open summoner tree");

    for (t, v) in times.iter().zip(&values) {
        let k = &t.to_ne_bytes();
        if let Ok(contains) = summoner_tree.contains_key(k) {
            if !contains {
                summoner_tree
                    .insert(k, &v.to_ne_bytes())
                    .expect("insert time, val");
            }
        }
    }

    // insert last scrape time into index
    if let Some(time) = times.last() {
        index.0
            .insert(summoner.as_bytes(), &time.to_ne_bytes())
            .unwrap();
    }

    // TODO preserve sorted order in database
    let mut time_vals: Vec<_> = summoner_tree
        .iter()
        .map(|r| r.expect("read"))
        .map(|(t, v)| (ivec_to_i64(t), ivec_to_i64(v)))
        .collect();
    time_vals.sort();

    let mut recent = recent.0.lock().expect("grab recent lock");
    recent.push_front(summoner);
    if recent.len() > RECENT_LEN {
        recent.pop_back();
    }

    let (times, values): (Vec<_>, Vec<_>) = time_vals.into_iter().unzip();
    Json(MatchInfo {
        ok: true,
        times,
        values,
    })
}

// #[derive(Serialize)]
// struct AddResp {
//     ok: bool,
// }
//
// #[get("/add/<summoner>")]
// async fn scraper(summoner: String) -> Json<AddResp> {
//     Json(AddResp { ok: true })
// }

#[derive(Serialize)]
struct TrackList {
    trackers: Vec<Tracker>,
}

#[derive(Serialize)]
struct Tracker {
    name: String,
    since: i64,
}

fn ivec_to_i64(v: sled::IVec) -> i64 {
    let v: Result<[u8; 8], _> = (*v).try_into();
    match v {
        Ok(v) => i64::from_ne_bytes(v),
        Err(_) => 0, // TODO
    }
}

#[get("/list")]
async fn list(recent: State<'_, Recent>, index: State<'_, Index>) -> Json<TrackList> {
    let recent = recent.0.lock().expect("grab recent lock");
    let v: Vec<Tracker> = recent.iter().map(|r| (r, index.0.get(r.as_bytes())))
        .map(|(k, r)| {
            let v = r.unwrap().unwrap();
            let v: Result<[u8; 8], _> = (*v).try_into();
            let since = match v {
                Ok(v) => i64::from_ne_bytes(v),
                Err(_) => 0,
            };
            Tracker {
                name: k.to_owned(),
                since,
            }
        }).collect();
    Json(TrackList { trackers: v })
}

#[launch]
fn rocket() -> rocket::Rocket {
    let index: sled::Db = sled::open("tracker_index").expect("open tracker_index database");
    let db = sled::open("league_db").expect("open league_db database");
    rocket::ignite()
        .manage(Recent(Arc::new(Mutex::new(VecDeque::new()))))
        .manage(Index(index))
        .manage(Db(db))
        .mount("/", routes![matches, list])
}
