use crate::fetcher::get_json_simple;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedFee {
    pub fastest_fee: u32,
    pub half_hour_fee: u32,
    pub hour_fee: u32,
    pub economy_fee: u32,
    pub minimum_fee: u32,
}

pub async fn get_recommended_fee() -> anyhow::Result<RecommendedFee> {
    let url = "https://mempool.space/api/v1/fees/recommended";
    let res = get_json_simple::<RecommendedFee>(url).await?;

    println!("rf: {:?}", res);

    Ok(res)
}
