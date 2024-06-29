use reqwest;
use serde_json;
use tokio;
use websocket;

#[tokio::main]
async fn main() {
    let params = [("username", "diane_bot"), ("password", "diane_bot")];
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let _res = client
        .post("http://localhost:5173/login")
        .form(&params)
        .send()
        .await;
    for game in serde_json::from_slice::<Vec<i64>>(
        client
            .get("http://localhost:5173/game")
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .as_ref(),
    )
    .unwrap()
    {}
}
