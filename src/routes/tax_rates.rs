use crate::db::*;
use crate::types::*;
use log::error;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;

#[get("/api/tax-rates?<world>")]
pub async fn tax_rates(
    mut db: Connection<TaxRates>,
    world: Option<u32>,
) -> Result<Json<TaxRatesValue>, Status> {
    match world {
        Some(w) => db.hgetall::<_, TaxRatesValue>(w).await.map_or_else(
            |e| {
                error!("{:?}", e);
                Err(Status::InternalServerError)
            },
            |tr| Ok(Json(tr)),
        ),
        None => Err(Status::NotFound),
    }
}

#[allow(non_snake_case)]
#[get("/api/v2/tax-rates?<world>")]
pub async fn tax_rates_v2(
    db: Connection<TaxRates>,
    world: Option<u32>,
) -> Result<Json<TaxRatesValue>, Status> {
    tax_rates(db, world).await
}
