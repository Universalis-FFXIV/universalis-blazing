use crate::db::*;
use crate::servers::*;
use crate::types::*;
use ironworks::excel::Excel;
use log::error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

#[allow(non_snake_case)]
#[get("/api/extra/stats/most-recently-updated?<world>&<dcName>")]
pub async fn most_recently_updated(
    mut db: Connection<Stats>,
    excel: &State<Excel<'_>>,
    world: Option<u32>,
    dcName: Option<&str>,
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    match world {
        Some(w) => match does_world_exist(excel, w) {
            Ok(r) => {
                if !r {
                    return Err(Status::NotFound);
                }

                db.zrevrange_withscores::<_, MostLeastRecentlyUpdated>(w, 0, -1)
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
                    )
            }
            Err(e) => {
                error!("{:?}", e);
                Err(Status::InternalServerError)
            }
        },
        None => Err(Status::NotFound),
    }
}

#[allow(non_snake_case)]
#[get("/api/v2/extra/stats/most-recently-updated?<world>&<dcName>")]
pub async fn most_recently_updated_v2(
    db: Connection<Stats>,
    excel: &State<Excel<'_>>,
    world: Option<u32>,
    dcName: Option<&str>,
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    most_recently_updated(db, excel, world, dcName).await
}
