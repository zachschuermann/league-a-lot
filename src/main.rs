use std::env;

#[async_std::main]
async fn main() -> surf::Result<()> {
    let API_KEY = env::var("RIOTAPIKEY").expect("RIOTAPIKEY environment variable set.");

    let mut res = surf::get("https://httpbin.org/get").await?;
    dbg!(res.body_string().await?);
    Ok(()) 
}
