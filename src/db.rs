use rocket_db_pools::{deadpool_redis, Database};

#[derive(Database)]
#[database("tax_rates")]
pub struct TaxRates(deadpool_redis::Pool);

#[derive(Database)]
#[database("stats")]
pub struct Stats(deadpool_redis::Pool);
