use futures::{stream, StreamExt};
use riven::consts::PlatformRoute;
use riven::RiotApiConfig;
use riven::{consts::RegionalRoute, models::match_v5::Match, RiotApi};
use serde::Serialize;

pub struct Client {
    riot: RiotApi,
}

#[derive(Serialize)]
pub struct TimeSeries {
    datetimes: Vec<i64>,
    playtimes: Vec<i64>,
}

impl Client {
    /// create a new client
    pub fn new() -> Self {
        let api_key = std::env::var("RIOTAPIKEY").expect("RIOTAPIKEY environment variable set.");
        let riot_config = RiotApiConfig::with_key(api_key);
        let riot = RiotApi::new(riot_config.preconfig_throughput());
        Client { riot }
    }

    /// get timeseries of match times for the given summoner
    pub async fn get_match_times(
        &self,
        summoner: &str,
    ) -> Result<TimeSeries, &dyn std::error::Error> {
        // 1. get summoner puuid
        // 2. get most recent 100 matches
        // 3. return two timeseries for match datetimes and playtimes
        let summoner = self
            .riot
            .summoner_v4()
            .get_by_summoner_name(PlatformRoute::NA1, summoner)
            .await
            .unwrap()
            .unwrap();
        let match_ids = self
            .riot
            .match_v5()
            .get_match_ids_by_puuid(
                RegionalRoute::AMERICAS,
                &summoner.puuid,
                Some(100),
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
        let matches: Vec<_> = stream::iter(match_ids)
            .map(|id| self.riot.match_v5().get_match(RegionalRoute::AMERICAS, &id))
            .buffer_unordered(100)
            .take(100)
            .collect()
            .await;
        let mut timeseries = matches
            .iter()
            .map(|m| {
                let m: &Match = m.as_ref().unwrap().as_ref().unwrap();
                (
                    m.info.game_creation / 1000,
                    match m.info.game_end_timestamp {
                        Some(_) => m.info.game_duration / 60,
                        None => m.info.game_duration / 60_000,
                    },
                )
            })
            .collect::<Vec<_>>();

        timeseries.sort_unstable();
        let (datetimes, playtimes) = timeseries.into_iter().unzip();

        Ok(TimeSeries {
            datetimes,
            playtimes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_match() {
        let riot = Client::new();
        let id = "NA1_4160719344";
        let match_ = riot
            .riot
            .match_v5()
            .get_match(RegionalRoute::AMERICAS, &id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(match_.metadata.match_id, id.to_string());
    }
}
