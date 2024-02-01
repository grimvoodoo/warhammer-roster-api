use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use tokio::fs;

use actix_web::{
    get, post,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct UnitQuery {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Unit {
    name: String,
    points: Vec<i32>,
    stats: Stats,
    weapons: Vec<Weapon>,
    abilities: HashMap<String, serde_json::Value>, // Using serde_json::Value for mixed types
    tags: Option<Vec<String>>,
    models: HashMap<String, Vec<i32>>,
    equipment: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    movement: i32,
    toughness: i32,
    save: i32,
    invulnerable: i32,
    wounds: i32,
    leadership: i32,
    objective_control: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Weapon {
    name: String,
    range: i32,
    attacks: i32,
    attack_dice: String,
    hit: i32,
    strength: i32,
    armour_pen: i32,
    damage: i32,
    tags: Option<Vec<String>>,
    ranged: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SimpleUnit {
    name: String,
    points: Vec<i32>,
}

// todo!("update the points to read from unit-points rahter than tyranids.json");

async fn tyranids() -> Result<Json<Vec<SimpleUnit>>, Box<dyn Error>> {
    let list = fs::read_to_string("data/tyranids/tyranids.json").await?;
    let data: Value = serde_json::from_str(&list)?;

    let mut units = Vec::new();

    if let Some(unit_list) = data.as_array() {
        for unit in unit_list {
            let name = unit["name"].as_str().unwrap_or_default().to_string();
            let points = unit["points"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .filter_map(|p| p.as_i64().map(|i| i as i32))
                .collect();
            units.push(SimpleUnit { name, points })
        }
    }
    Ok(Json(units))
}

async fn tyranids_unit(query_string: String) -> Result<Json<SimpleUnit>, Box<dyn Error>> {
    let list = fs::read_to_string("data/tyranids/tyranids.json").await?;
    let data: Value = serde_json::from_str(&list)?;

    if let Some(unit_list) = data.as_array() {
        for unit in unit_list {
            if unit["name"].as_str().unwrap_or_default() == query_string {
                let name = unit["name"].as_str().unwrap_or_default().to_string();
                let points = unit["points"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|p| p.as_i64().map(|i| i as i32))
                    .collect();

                return Ok(Json(SimpleUnit { name, points }));
            }
        }
    }

    // Returning an error if no matching unit is found
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Unit not found",
    )))
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(
        "
        Welcome to the Warhammer 40k army roster API. Here are instructions on how to use this API:
        To see a full list of all the army units then add a suffix of the army name. 
        For example `/tyranids` will return a list of all the tyranid units
        ",
    )
}

#[get("/tyranids")]
async fn tyranids_get_list() -> impl Responder {
    match tyranids().await {
        Ok(units) => HttpResponse::Ok().json(units), // Respond with JSON
        Err(e) => {
            eprintln!("Failed to read units: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to read units: {}", e))
        }
    }
}

#[get("/unit")]
async fn get_unit(query: web::Query<UnitQuery>) -> impl Responder {
    let unit_name = &query.name;

    match tyranids_unit(unit_name.clone()).await {
        Ok(unit) => HttpResponse::Ok().json(unit), // If found, return the unit
        Err(_) => HttpResponse::NotFound().body(format!("No unit found with name: {}", unit_name)), // If not found or an error occurs
    }
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(tyranids_get_list)
            .service(get_unit)
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
