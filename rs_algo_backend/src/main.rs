use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::io::Result;

mod db;
mod error;
mod middleware;
mod models;
mod render_chart;
mod services;
mod strategies;

use db::mongo;
use error::RsAlgoError;
use middleware::cors::cors_middleware;
use middleware::logger::logger_middleware;
use models::app_state::AppState;
use models::db::Db;
use services::back_test;
use services::bot;
use services::index::index;
use services::instrument;
use services::portfolio;
use services::watch_list;
use std::env;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("BACKEND_PORT").expect("BACKEND_PORT not found");
    let app_name = env::var("BACKEND_NAME").expect("BACKEND_NAME not found");

    let username = env::var("DB_USERNAME").expect("DB_USERNAME not found");
    let password = env::var("DB_PASSWORD").expect("DB_PASSWORD not found");
    let db_mem_name = env::var("MONGO_MEM_DB_NAME").expect("MONGO_MEM_DB_NAME not found");
    let db_mem_uri = env::var("MONGO_MEM_DB_URI").expect("MONGO_MEM_DB_URI not found");

    let db_hdd_name = env::var("MONGO_HDD_DB_NAME").expect("MONGO_HD_DB_NAME not found");
    let db_hdd_uri = env::var("MONGO_HDD_DB_URI").expect("MONGO_HD_DB_URI not found");

    let db_bot_name = env::var("MONGO_BOT_DB_NAME").expect("MONGO_BOT_DB_NAME not found");
    let db_bot_uri = env::var("MONGO_BOT_DB_URI").expect("MONGO_BOT_DB_URI not found");

    let mongodb_mem_client: mongodb::Client =
        mongo::connect(&username, &password, &db_mem_name, &db_mem_uri)
            .await
            .map_err(|_e| RsAlgoError::NoDbConnection)
            .unwrap();

    let mongodb_hdd_client: mongodb::Client =
        mongo::connect(&username, &password, &db_hdd_name, &db_hdd_uri)
            .await
            .map_err(|_e| RsAlgoError::NoDbConnection)
            .unwrap();

    let mongodb_bot_client: mongodb::Client =
        mongo::connect(&username, &password, &db_bot_name, &db_bot_uri)
            .await
            .map_err(|_e| RsAlgoError::NoDbConnection)
            .unwrap();

    log::info!("Starting {} on port {} !", app_name, port.clone());
    let payload_limit = 1024 * 1024 * 64;

    HttpServer::new(move || {
        App::new()
            .wrap(cors_middleware())
            .wrap(logger_middleware())
            .data(AppState {
                app_name: String::from(&app_name),
                db_bot: Db {
                    client: mongodb_bot_client.clone(),
                    name: db_bot_name.to_owned(),
                },
                db_mem: Db {
                    client: mongodb_mem_client.clone(),
                    name: db_mem_name.to_owned(),
                },
                db_hdd: Db {
                    client: mongodb_hdd_client.clone(),
                    name: db_hdd_name.to_owned(),
                },
            })
            .app_data(web::PayloadConfig::new(payload_limit))
            .app_data(web::JsonConfig::default().limit(payload_limit))
            .route("/", web::get().to(index))
            .service(
                web::scope("/api")
                    .route("/bots", web::get().to(bot::find))
                    .route("/bots/chart/{id}", web::get().to(bot::chart))
                    .route("/instruments", web::post().to(instrument::find))
                    .route("/instruments", web::put().to(instrument::upsert))
                    .route("/instruments/{symbol}", web::get().to(instrument::find_one))
                    .route(
                        "/instruments/chart/{symbol}",
                        web::get().to(instrument::chart),
                    )
                    .route("/watchlist", web::get().to(watch_list::find))
                    .route("/watchlist", web::put().to(watch_list::upsert))
                    .route("/watchlist", web::delete().to(watch_list::delete))
                    .route("/portfolio", web::get().to(portfolio::find))
                    .route("/portfolio", web::put().to(portfolio::upsert))
                    .route("/portfolio", web::delete().to(portfolio::delete))
                    .route(
                        "/backtest",
                        web::put().to(back_test::upsert_instruments_result),
                    )
                    .route(
                        "/backtest/instruments/{symbol}/{time_frame}",
                        web::get().to(back_test::find_one),
                    )
                    .route(
                        "/backtest/instruments/compact",
                        web::get().to(back_test::find_compact_instruments),
                    )
                    .route(
                        "/backtest/instruments/markets/{market}/{time_frame}",
                        web::get().to(back_test::find_instruments),
                    )
                    .route(
                        "/backtest/strategies/instrument/{instrument}",
                        web::get().to(back_test::find_strategies_result_instruments),
                    )
                    .route(
                        "/backtest/strategies",
                        web::get().to(back_test::find_strategies_result),
                    )
                    .route(
                        "/backtest/strategies",
                        web::put().to(back_test::upsert_strategies_result),
                    )
                    .route("/backtest/prices", web::get().to(back_test::find_prices))
                    .route(
                        "/backtest/strategies/{uuid}",
                        web::get().to(back_test::find_instruments_result_by_strategy),
                    )
                    .route(
                        "/backtest/strategies/chart/{uuid}/{symbol}",
                        web::get().to(back_test::chart),
                    ),
            )
    })
    .bind(["0.0.0.0:", &port].concat())
    .expect("[BACKEND ERROR] Can't launch server!")
    .run()
    .await
}
