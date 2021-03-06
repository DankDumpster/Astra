use crate::services::Db;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct DBLaunch {
    pub launch_id: String,
    pub name: String,
    pub net: DateTime<Utc>,
    pub vid_url: Option<String>,
    pub image_url: Option<String>,
    pub dispatched: bool,
    pub status: i32,
    pub description: Option<String>,
}

impl Db {
    pub async fn get_launches(&self) -> Vec<DBLaunch> {
        sqlx::query_as!(
            DBLaunch,
            "SELECT * FROM astra.launches WHERE net > now() ORDER BY net"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    pub async fn get_limited_launches(&self) -> Vec<DBLaunch> {
        sqlx::query_as!(
            DBLaunch,
            "SELECT * FROM astra.launches WHERE net <= (now() + interval '24 hours') AND status = 1;"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
}
