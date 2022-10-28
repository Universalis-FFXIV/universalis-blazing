#[macro_use]
extern crate rocket;

mod db;
mod fairings;
mod routes;
mod types;

use crate::db::*;
use crate::fairings::metrics::*;
use crate::routes::least_recently_updated::*;
use crate::routes::metrics::*;
use crate::routes::most_recently_updated::*;
use crate::routes::recently_updated::*;
use crate::routes::tax_rates::*;
use rocket::fairing::AdHoc;
use rocket_db_pools::Database;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Stats::init())
        .attach(TaxRates::init())
        .attach(Metrics::new().expect("Failed to initialize metrics system"))
        .attach(AdHoc::on_liftoff("Ready Checker", |_| {
            Box::pin(async {
                println!("Application started");
            })
        }))
        .mount(
            "/",
            routes![
                metrics,
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
