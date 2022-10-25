#[macro_use]
extern crate rocket;

mod db;
mod routes;
mod types;

use crate::db::*;
use crate::routes::least_recently_updated::*;
use crate::routes::most_recently_updated::*;
use crate::routes::recently_updated::*;
use crate::routes::tax_rates::*;
use rocket_db_pools::Database;

// TODO: map redis errors in endpoints to other status codes

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
