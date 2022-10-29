use std::collections::HashMap;

use rocket_db_pools::deadpool_redis::redis::{self, FromRedisValue, RedisResult};
use serde::Serialize;

#[derive(Serialize)]
pub struct TaxRatesValue {
    #[serde(rename(serialize = "Limsa Lominsa"))]
    pub limsa_lominsa: u8,
    #[serde(rename(serialize = "Gridania"))]
    pub gridania: u8,
    #[serde(rename(serialize = "Ul'dah"))]
    pub uldah: u8,
    #[serde(rename(serialize = "Ishgard"))]
    pub ishgard: u8,
    #[serde(rename(serialize = "Kugane"))]
    pub kugane: u8,
    #[serde(rename(serialize = "Crystarium"))]
    pub crystarium: u8,
    #[serde(rename(serialize = "Old Sharlayan"))]
    pub old_sharlayan: u8,
}

#[derive(Serialize)]
pub struct RecentlyUpdated {
    pub items: Vec<u32>,
}

#[derive(Serialize, Clone, Copy)]
pub struct WorldItemUpload {
    #[serde(rename(serialize = "worldId"))]
    pub world_id: u32,
    #[serde(rename(serialize = "itemId"))]
    pub item_id: u32,
    #[serde(rename(serialize = "lastUploadTime"))]
    pub last_upload_time: i64,
}

impl WorldItemUpload {
    pub fn new() -> Self {
        Self {
            world_id: 0,
            item_id: 0,
            last_upload_time: 0,
        }
    }
}

#[derive(Serialize)]
pub struct MostLeastRecentlyUpdated {
    pub items: Vec<WorldItemUpload>,
}

pub fn try_get_int(v: &redis::Value) -> Option<i64> {
    match v {
        redis::Value::Int(n) => Some(*n),
        redis::Value::Data(buf) => String::from_utf8(buf.clone())
            .ok()
            .map_or(None, |s| s.parse::<i64>().ok()),
        _ => None,
    }
}

pub fn hash_to_map(values: &Vec<redis::Value>) -> RedisResult<HashMap<String, redis::Value>> {
    let mut map = HashMap::new();
    let mut last_key = String::new();
    for i in 0..values.len() {
        if i % 2 == 0 {
            last_key = String::from_redis_value(&values[i])?;
            map.insert(last_key.clone(), redis::Value::Nil);
        } else {
            map.insert(last_key.clone(), values[i].clone());
        }
    }
    Ok(map)
}

pub fn redis_map_get_i64(map: &HashMap<String, redis::Value>, key: String) -> i64 {
    try_get_int(&map.get(&key).unwrap_or(&redis::Value::Nil)).unwrap_or_default()
}

impl FromRedisValue for TaxRatesValue {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Bulk(values) => {
                let map = hash_to_map(values)?;
                if map.len() < 7 {
                    Err((
                        redis::ErrorKind::TypeError,
                        "Expected at least 6 elements in response, got fewer",
                    )
                        .into())
                } else {
                    Ok(TaxRatesValue {
                        limsa_lominsa: redis_map_get_i64(&map, "Limsa Lominsa".to_string()) as u8,
                        gridania: redis_map_get_i64(&map, "Gridania".to_string()) as u8,
                        uldah: redis_map_get_i64(&map, "Ul'dah".to_string()) as u8,
                        ishgard: redis_map_get_i64(&map, "Ishgard".to_string()) as u8,
                        kugane: redis_map_get_i64(&map, "Kugane".to_string()) as u8,
                        crystarium: redis_map_get_i64(&map, "Crystarium".to_string()) as u8,
                        old_sharlayan: redis_map_get_i64(&map, "Old Sharlayan".to_string()) as u8,
                    })
                }
            }
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
