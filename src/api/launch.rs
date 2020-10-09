use crate::api::common::{ApiResult, Status};
use crate::api::mission::Mission;
use crate::api::rocket::Rocket;
use crate::api::url::VidURL;
use crate::api::BASE_URL;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize, Serialize, Debug)]
pub struct Launch {
    pub id: String,
    pub url: String,
    // pub launch_library_id: i32,
    pub slug: String,
    pub name: String,
    pub status: Status,
    pub net: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub inhold: bool,
    pub tbdtime: bool,
    pub tbddate: bool,
    pub probability: Option<i8>,
    pub holdreason: Option<String>,
    pub failreason: Option<String>,
    pub rocket: Rocket,
    pub mission: Option<Mission>,
    // #[serde(alias = "infoURLs")]
    // pub info_urls: Vec<String>,
    #[serde(alias = "vidURLs")]
    pub vid_urls: Vec<VidURL>,
}

pub async fn get_next_launch<'a>() -> Result<ApiResult<Launch>, Box<dyn Error>> {
    let res = reqwest::get(&format!(
        "{}/launch/upcoming/?format=json&mode=detailed",
        BASE_URL
    ))
    .await?
    .json::<ApiResult<Launch>>()
    .await?;

    Ok(res)
}
