use reqwest::Client;
use serde::Deserialize;

pub async fn get_json_simple<T: for<'de> Deserialize<'de>>(url: &str) -> anyhow::Result<T> {
    let client = Client::new();

    let res = client
        .get(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .json::<T>()
        .await?;

    Ok(res)
}
