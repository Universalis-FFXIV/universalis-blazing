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

#[get("/api/extra/stats/recently-updated?<world>")]
pub async fn recently_updated(
    mut db: Connection<Stats>,
    excel: &State<Excel<'_>>,
    world: Option<u32>,
) -> Result<Json<RecentlyUpdated>, Status> {
    match world {
        Some(w) => match does_world_exist(excel, w) {
            Ok(r) => {
                if !r {
                    return Err(Status::NotFound);
                }

                db.zrange::<_, RecentlyUpdated>(w, 0, -1).await.map_or_else(
                    |e| {
                        error!("{:?}", e);
                        Err(Status::InternalServerError)
                    },
                    |ru| Ok(Json(ru)),
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
#[get("/api/v2/extra/stats/recently-updated?<world>")]
pub async fn recently_updated_v2(
    db: Connection<Stats>,
    excel: &State<Excel<'_>>,
    world: Option<u32>,
) -> Result<Json<RecentlyUpdated>, Status> {
    recently_updated(db, excel, world).await
}
