use crate::db::*;
use crate::types::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

#[get("/api/extra/stats/recently-updated?<world>")]
pub async fn recently_updated(
    mut db: Connection<Stats>,
    world: Option<u32>,
) -> Result<Json<RecentlyUpdated>, Status> {
    match world {
        Some(w) => db
            .zrange::<_, RecentlyUpdated>(w, 0, -1)
            .await
            .map_or_else(|_| Err(Status::NotFound), |ru| Ok(Json(ru))),
        None => Err(Status::NotFound),
    }
}

#[allow(non_snake_case)]
#[get("/api/v2/extra/stats/recently-updated?<world>")]
pub async fn recently_updated_v2(
    db: Connection<Stats>,
    world: Option<u32>,
) -> Result<Json<RecentlyUpdated>, Status> {
    recently_updated(db, world).await
}
