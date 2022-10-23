#[macro_use]
extern crate rocket;

mod types;

use crate::types::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::{Connection, Database};

// TODO: map redis errors in endpoints to other status codes

// Rocket doesn't have support for aliasing query parameters, so I need to
// just ignore the warning in order to not break API contracts.
#[allow(non_snake_case)]
#[get("/api/extra/stats/least-recently-updated?<world>&<dcName>")]
async fn least_recently_updated(
    mut db: Connection<Stats>,
    world: Option<u32>,
    dcName: Option<&str>, // TODO: DC support?
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    match world {
        Some(w) => db
            .zrange_withscores::<_, MostLeastRecentlyUpdated>(w, 0, -1)
            .await
            .map_or_else(
                |_| Err(Status::NotFound),
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
async fn least_recently_updated_v2(
    db: Connection<Stats>,
    world: Option<u32>,
    dcName: Option<&str>,
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    least_recently_updated(db, world, dcName).await
}

#[allow(non_snake_case)]
#[get("/api/extra/stats/most-recently-updated?<world>&<dcName>")]
async fn most_recently_updated(
    mut db: Connection<Stats>,
    world: Option<u32>,
    dcName: Option<&str>,
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    match world {
        Some(w) => db
            .zrevrange_withscores::<_, MostLeastRecentlyUpdated>(w, 0, -1)
            .await
            .map_or_else(
                |_| Err(Status::NotFound),
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
#[get("/api/v2/extra/stats/most-recently-updated?<world>&<dcName>")]
async fn most_recently_updated_v2(
    db: Connection<Stats>,
    world: Option<u32>,
    dcName: Option<&str>,
) -> Result<Json<MostLeastRecentlyUpdated>, Status> {
    most_recently_updated(db, world, dcName).await
}

#[get("/api/extra/stats/recently-updated?<world>")]
async fn recently_updated(
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
async fn recently_updated_v2(
    db: Connection<Stats>,
    world: Option<u32>,
) -> Result<Json<RecentlyUpdated>, Status> {
    recently_updated(db, world).await
}

#[get("/api/tax-rates?<world>")]
async fn tax_rates(
    mut db: Connection<TaxRates>,
    world: Option<u32>,
) -> Result<Json<TaxRatesValue>, Status> {
    match world {
        Some(w) => db
            .hgetall::<_, TaxRatesValue>(w)
            .await
            .map_or_else(|_| Err(Status::NotFound), |tr| Ok(Json(tr))),
        None => Err(Status::NotFound),
    }
}

#[allow(non_snake_case)]
#[get("/api/v2/tax-rates?<world>")]
async fn tax_rates_v2(
    db: Connection<TaxRates>,
    world: Option<u32>,
) -> Result<Json<TaxRatesValue>, Status> {
    tax_rates(db, world).await
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Stats::init())
        .attach(TaxRates::init())
        .mount(
            "/",
            routes![
                least_recently_updated,
                least_recently_updated_v2,
                most_recently_updated,
                most_recently_updated_v2,
                recently_updated,
                recently_updated_v2,
                tax_rates,
                tax_rates_v2,
            ],
        )
}
