#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::serde::{json::Json, Serialize};
use rocket_db_pools::deadpool_redis::redis::{self, AsyncCommands, FromRedisValue};
use rocket_db_pools::{deadpool_redis, Connection, Database};

#[derive(Database)]
#[database("tax_rates")]
struct TaxRates(deadpool_redis::Pool);

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TaxRatesValue {
    limsa_lominsa: u8,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct RecentlyUpdated {
    items: Vec<u32>,
}

#[derive(Serialize, Clone, Copy)]
#[serde(crate = "rocket::serde")]
struct WorldItemUpload {
    world_id: u32,
    item_id: u32,
    last_upload_time: i64,
}

impl WorldItemUpload {
    fn new() -> Self {
        Self {
            world_id: 0,
            item_id: 0,
            last_upload_time: 0,
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct MostLeastRecentlyUpdated {
    items: Vec<WorldItemUpload>,
}

fn try_get_int(v: &redis::Value) -> Option<i64> {
    match v {
        redis::Value::Int(n) => Some(*n),
        _ => None,
    }
}

impl FromRedisValue for TaxRatesValue {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Bulk(values) => Ok(TaxRatesValue {
                limsa_lominsa: try_get_int(&values[0]).unwrap_or_default() as u8,
            }),
            _ => Err((
                redis::ErrorKind::TypeError,
                "Expected bulk response, got other response type",
            )
                .into()),
        }
    }
}

impl FromRedisValue for RecentlyUpdated {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Bulk(values) => Ok(RecentlyUpdated {
                items: values
                    .into_iter()
                    .filter_map(try_get_int)
                    .map(|n| n as u32)
                    .collect(),
            }),
            _ => Err((
                redis::ErrorKind::TypeError,
                "Expected bulk response, got other response type",
            )
                .into()),
        }
    }
}

impl FromRedisValue for MostLeastRecentlyUpdated {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Bulk(values) => {
                let mut items = vec![WorldItemUpload::new(); values.len() / 2];
                let mut iter = values.clone().into_iter();
                for i in 0..items.len() {
                    let item_id_opt = iter.next().and_then(|v| try_get_int(&v));
                    let timestamp_opt = iter.next().and_then(|v| try_get_int(&v));
                    if let Some((item_id, timestamp)) = item_id_opt.zip(timestamp_opt) {
                        items[i] = WorldItemUpload {
                            world_id: 0,
                            item_id: item_id as u32,
                            last_upload_time: timestamp,
                        };
                    }
                }
                Ok(MostLeastRecentlyUpdated { items })
            }
            _ => Err((
                redis::ErrorKind::TypeError,
                "Expected bulk response, got other response type",
            )
                .into()),
        }
    }
}

// Rocket doesn't have support for aliasing query parameters, so I need to
// just ignore the warning in order to not break API contracts.
#[allow(non_snake_case)]
#[get("/api/extra/stats/least-recently-updated?<world>&<dcName>")]
async fn least_recently_updated(
    mut db: Connection<TaxRates>,
    world: Option<u32>,
    dcName: Option<&str>, // TODO: DC support>
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
#[get("/api/extra/stats/most-recently-updated?<world>&<dcName>")]
async fn most_recently_updated(
    mut db: Connection<TaxRates>,
    world: Option<u32>,
    dcName: Option<&str>, // TODO: DC support>
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

#[get("/api/extra/stats/recently-updated?<world>")]
async fn recently_updated(
    mut db: Connection<TaxRates>,
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

#[launch]
fn rocket() -> _ {
    rocket::build().attach(TaxRates::init()).mount(
        "/",
        routes![
            least_recently_updated,
            most_recently_updated,
            recently_updated,
            tax_rates
        ],
    )
}
