use crate::db::*;
use crate::types::*;
use log::error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

// Rocket doesn't have support for aliasing query parameters, so I need to
// just ignore the warning in order to not break API contracts.
#[allow(non_snake_case)]
#[get("/api/extra/stats/least-recently-updated?<world>&<dcName>")]
pub async fn least_recently_updated(
    mut db: Connection<Stats>,
    world: Option<u32>,
    dcName: Option<&str>, // TODO: DC support?
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    match world {
        Some(w) => db
            .zrange_withscores::<_, MostLeastRecentlyUpdated>(w, 0, -1)
            .await
            .map_or_else(
                |e| {
                    error!("{:?}", e);
                    Err(Status::InternalServerError)
                },
                |ru| {
                    Ok(Json(MostLeastRecentlyUpdated {
                        items: ru
                            .items
                            .into_iter()
                            .map(|item| WorldItemUpload {
                                world_id: w,
                                ..item
                            })
                            .collect::<Vec<WorldItemUpload>>(),
                    }))
                },
            ),
        None => Err(Status::NotFound),
    }
}

#[allow(non_snake_case)]
#[get("/api/v2/extra/stats/least-recently-updated?<world>&<dcName>")]
pub async fn least_recently_updated_v2(
    db: Connection<Stats>,
    world: Option<u32>,
    dcName: Option<&str>,
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    least_recently_updated(db, world, dcName).await
}
