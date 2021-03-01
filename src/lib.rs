use riven::consts::Region;
use riven::{
    RiotApi,
    models::match_v4::{Match, MatchReference},
};
use futures::future::join_all;
use once_cell::sync::OnceCell;

pub static RIOT_API: OnceCell<RiotApi> = OnceCell::new();

pub async fn get_all(matches: Vec<MatchReference>) -> (Vec<i64>, Vec<i64>) {
    let match_futures = matches
        .iter()
        .map(|m| get_match(m.game_id))
        .collect::<Vec<_>>();
    let matches = join_all(match_futures).await;
    let (mut times, mut values): (Vec<_>, Vec<_>) = matches
        .iter()
        .map(|m| {
            match m.as_ref() {
                Ok(match_) => Some(match_),
                Err(e) => {
                    println!("get match error: {}", e);
                    None
                },
            }
        })
        .flatten() 
        .map(|m| (m.game_creation / 1000, m.game_duration / 60))
        .unzip();
    times.reverse();
    values.reverse();
    (times, values)
}

pub async fn get_match_history_since(summoner: &str, since: i64) -> Result<Vec<MatchReference>, &str> {
    let name = summoner;
    let summoner = RIOT_API.get().expect("riot api init")
        .summoner_v4()
        .get_by_summoner_name(Region::NA, summoner)
        .await;

    // TODO ugly
    if summoner.is_err() {
        return Err("failed get summoner request");
    }
    let summoner = summoner.unwrap();
    if summoner.is_none() {
        return Err("no summoner found");
    }
    let summoner = summoner.unwrap();

    let mut matches = vec![];
    let match_list = RIOT_API.get().expect("riot api init")
        .match_v4()
        .get_matchlist(
            Region::NA,
            &summoner.account_id,
            Some(since * 1000 + 1), // need timestamp in ms
            None, // Some(0),
            None,
            None, // Some(start + 604800000),
            None,
            None,
            None,
        )
        .await;
    // .expect("Get matchlist failed.")
    // .expect("No matchlist for account id.");
    // TODO ugly
    if match_list.is_err() {
        return Err("Get matchlist failed.");
    }
    let match_list = match_list.unwrap();
    if match_list.is_none() {
        println!("no new matches for {} since {}", name, since);
        return Ok(vec![]);
    }
    let mut match_list = match_list.unwrap();
    if match_list.matches.len() > 99 {
        println!("over 100 found. scraping entire history.");
        return get_match_history(name).await;
    }
    matches.append(&mut match_list.matches);

    println!("found {} matches for {} since {}", matches.len(), name, since);
    Ok(matches)
}

pub async fn get_match_history(summoner: &str) -> Result<Vec<MatchReference>, &str> {
    let name = summoner;
    let summoner = RIOT_API.get().expect("riot api init")
        .summoner_v4()
        .get_by_summoner_name(Region::NA, summoner)
        .await;

    // TODO ugly
    if summoner.is_err() {
        return Err("failed get summoner request");
    }
    let summoner = summoner.unwrap();
    if summoner.is_none() {
        return Err("no summoner found");
    }
    let summoner = summoner.unwrap();

    // println!("\n{}\n", &summoner.account_id);

    let mut start_index = 0;
    let mut matches = vec![];
    loop {
        let match_list = RIOT_API.get().expect("riot api init")
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
            .await;

        // TODO ugly
        if match_list.is_err() {
            return Err("Get matchlist failed.");
        }
        let match_list = match_list.unwrap();
        if match_list.is_none() {
            return Err("No matchlist for account id.");
        }
        let mut match_list = match_list.unwrap();
        if match_list.matches.is_empty() {
            break;
        }
        matches.append(&mut match_list.matches);
        start_index += 100;
    }

    println!("found {} matches for {}", matches.len(), name);
    Ok(matches)
}

pub async fn get_match(game_id: i64) -> Result<Match, &'static str> {
    // do x retries max
    //for n in 1..=MAX_RETRIES {
        //println!("try {}", n);
    let match_ = RIOT_API.get().expect("riot api init").match_v4().get_match(Region::NA, game_id).await;

    if let Ok(ok_match) = match_ {
        if let Some(m) = ok_match {
            return Ok(m);
        }
    }
        // backoff
        //sleep(Duration::from_secs(n.pow(2) as u64)).await;
    //}

    Err("Failed to get match")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_since() {
        let matches = get_match_history_since("test", 1609522696000).await.expect("this works");
        assert_eq!(matches.len(), 10);
    }
}
