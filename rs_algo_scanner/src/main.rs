use crate::broker::Symbol;
use crate::error::Result;
use error::RsAlgoErrorKind;
use std::time::Instant;

use broker::xtb::*;
use instrument::Instrument;
use rs_algo_shared::helpers::date;
use rs_algo_shared::helpers::date::Local;
use rs_algo_shared::models::TimeFrame;
use screener::Screener;

mod backend;
mod broker;
mod candle;
mod error;
mod helpers;
mod indicators;
mod instrument;
mod patterns;
mod prices;
mod screener;

use dotenv::dotenv;
use rs_algo_shared::helpers::http::{request, HttpMethod};

use std::env;
use std::{thread, time};
/*
TODO LIST
- Calculate % of the pattern
- Add activated chart figures for channels and broadenings
- Fix horizontal levels
- Calculate divergences on indicators
- Review candles formulas
- Add degrees to higher_highs increment/decrement
*/

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let start = Instant::now();
    let username = &env::var("BROKER_USERNAME").unwrap();
    let password = &env::var("BROKER_PASSWORD").unwrap();
    let from_date = env::var("FROM_DATE").unwrap().parse::<i64>().unwrap();
    let sleep_time = &env::var("SLEEP_TIME").unwrap().parse::<u64>().unwrap();
    let time_frame = &env::var("TIME_FRAME").unwrap();

    let sleep = time::Duration::from_millis(*sleep_time);
    let from = (Local::now() - date::Duration::days(from_date)).timestamp();

    let time_frame = TimeFrame::new(time_frame);

    let mut screener = Screener::<Xtb>::new().await?;
    screener.login(username, password).await?;
    let symbols = screener.get_symbols().await.unwrap().symbols;

    // let symbols = [
    //     Symbol {
    //         symbol: "ESV.US_9".to_owned(),
    //         category: "".to_owned(),
    //         description: "".to_owned(),
    //         currency: "".to_owned(),
    //     },
    //     // Symbol {
    //     //     symbol: "BUD.US_9".to_owned(),
    //     //     category: "".to_owned(),
    //     //     description: "".to_owned(),
    //     //     currency: "".to_owned(),
    //     // },
    // ];

    let ignore_list: Vec<String> = env::var("SYMBOL_IGNORE_LIST")
        .unwrap()
        .split('@')
        .map(|x| x.to_owned())
        .collect();

    for s in symbols {
        let now = Instant::now();
        println!("[INSTRUMENT] {:?} processing...", &s.symbol);
        if !ignore_list.contains(&s.symbol) {
            screener
                .get_instrument_data(
                    &s.symbol,
                    time_frame.clone(),
                    from,
                    |instrument: Instrument| async move {
                        println!(
                            "[INSTRUMENT] {:?} processed in {:?}",
                            &instrument.symbol(),
                            now.elapsed()
                        );

                        let endpoint = env::var("BACKEND_INSTRUMENTS_ENDPOINT").unwrap().clone();
                        let now = Instant::now();

                        let res = request::<Instrument>(&endpoint, &instrument, HttpMethod::Put)
                            .await
                            .map_err(|_e| RsAlgoErrorKind::RequestError)?;

                        println!(
                            "[RESPONSE] {:?} status {:?} at {:?} in {:?}",
                            &instrument.symbol(),
                            res.status(),
                            Local::now(),
                            now.elapsed()
                        );

                        Ok(())
                    },
                )
                .await?;
            thread::sleep(sleep);
        }
    }

    println!("[Finished] at {:?}  in {:?}", Local::now(), start.elapsed());

    Ok(())
}