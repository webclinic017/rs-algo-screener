use super::pattern_status::get_pattern_status;
use async_trait::async_trait;
use bson::{doc, Document};
use chrono::Duration;
use futures::stream::StreamExt;
use mongodb::Cursor;
use rs_algo_shared::models::divergence::{Divergence, DivergenceType};
use rs_algo_shared::models::instrument::*;
use rs_algo_shared::models::pattern::{DataPoints, Pattern, PatternType};
use rs_algo_shared::models::status::Status;
use std::cmp::Ordering;

use round::round;
use rs_algo_shared::error::Result;
use rs_algo_shared::helpers::comp::*;
use rs_algo_shared::helpers::date::*;
use std::env;

pub struct General {
    pub query: Document,
    pub max_pattern_date: DbDateTime,
    pub max_activated_date: DbDateTime,
}

//FIMXE impl trait (fix asyn-trait)
impl General {
    pub fn new() -> Result<General> {
        let max_pattern_days = env::var("MAX_PATTERN_DAYS")
            .unwrap()
            .parse::<i64>()
            .unwrap();

        let max_pattern_activated_days = env::var("MAX_PATTERN_ACTIVATED_DAYS")
            .unwrap()
            .parse::<i64>()
            .unwrap();

        Ok(Self {
            query: doc! {},
            max_pattern_date: to_dbtime(Local::now() - Duration::days(max_pattern_days)),
            max_activated_date: to_dbtime(
                Local::now() - Duration::days(max_pattern_activated_days),
            ),
        })
    }

    pub fn query(&self) -> Document {
        let minimum_pattern_target = env::var("MINIMUM_PATTERN_TARGET")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        let min_horizontal_level_ocurrences = env::var("MIN_HORIZONTAL_LEVELS_OCCURENCES")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        doc! {
        "$or": [
            {"$or": [
                {"$and": [
                    {"patterns.local_patterns": {"$elemMatch" : {
                    "date": { "$gte" : self.max_pattern_date },
                    "pattern_type": { "$in": ["ChannelUp","TriangleUp","Rectangle","BroadeningUp","DoubleBottom","HeadShoulders"] },
                    }}}
                ]},
                {"$and": [
                    {"patterns.local_patterns": {"$elemMatch" : {
                    "active.target":{"$gte": minimum_pattern_target },
                    "active.date": { "$gte" : self.max_activated_date },
                    "pattern_type": { "$in": ["ChannelUp","TriangleUp","Rectangle","BroadeningUp","DoubleBottom","HeadShoulders"] },
                    }}}
                ]},
                // {"$and": [
                //     {"divergences.data": {"$elemMatch" : {
                //         //"date": { "$gte" : self.max_pattern_date },
                //         "divergence_type": { "$in": ["Bullish","Bearish"] } ,
                //     }}}
                // ]},
                // {"$and": [
                //     {"patterns.extrema_patterns": {"$elemMatch" : {
                //     "active.target":{"$gte": minimum_pattern_target },
                //     "date": { "$gte" : self.max_pattern_date },
                //     }}}
                // ]},
                // {"$and": [
                //     {"patterns.extrema_patterns": {"$elemMatch" : {
                //     "active.target":{"$gte": minimum_pattern_target },
                //     "active.date": { "$gte" : self.max_activated_date }
                //     }}}
                // ]},
                ]
            },
        //     {"$and": [
        //         {"$expr": {"$gte": ["$indicators.bb_a.current_a","$indicators.bb_b.current_a"]}},
        //         //{"$expr": {"$lte": ["$indicators.bb_a.current_a","$indicators.bb_b.prev_a"]}},
        //         {"$expr": {"$lt": ["$indicators.bb_a.prev_a","$indicators.bb_b.prev_a"]}},
        //         //{"$expr": {"$gte": ["$indicators.bb_b.current_a","$indicators.ema_c.current_a"]}},
        //    ]},
            { "symbol": { "$in": [ "BITCOIN","ETHEREUM","RIPPLE","DOGECOIN","POLKADOT","STELLAR","CARDANO","SOLANA"] } },
            { "symbol": { "$in": [ "US500","US100","GOLD","OIL","SILVER"] } },
            //{ "current_candle": { "$in": ["Karakasa","BullishGap","MorningStar"] } },
            // {"$and": [
            //  {
            //     "horizontal_levels.lows": {"$elemMatch" : {
            //    // "price":{"$gte": "$current_price" },
            //     "occurrences":{"$gte": min_horizontal_level_ocurrences },
            //     }}},
            // ]},
        ]}
    }

    pub async fn format_instrument(
        &self,
        mut instruments: Cursor<CompactInstrument>,
    ) -> Vec<CompactInstrument> {
        println!("[STRATEGY] Formating ");
        let mut docs: Vec<CompactInstrument> = vec![];

        let bb_crossover_th = env::var("bb_cROSSOVER_THRESHOLD")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        // let minimum_pattern_target = env::var("MINIMUM_PATTERN_TARGET")
        //     .unwrap()
        //     .parse::<f64>()
        //     .unwrap();

        while let Some(result) = instruments.next().await {
            match result {
                Ok(mut instrument) => {
                    //MOVE THIS TO SHARED
                    let stoch = instrument.indicators.stoch.clone();
                    let macd = instrument.indicators.macd.clone();
                    let rsi = instrument.indicators.rsi.clone();
                    let bb = instrument.indicators.bb.clone(); //8

                    //let ema_c = instrument.indicators.ema_c.clone(); //55
                    let len = instrument.patterns.local_patterns.len();
                    let last_pattern = instrument.patterns.local_patterns.last();

                    let last_divergence = instrument.divergences.data.last();

                    let last_pattern_target = match last_pattern {
                        Some(val) => round(val.active.change, 0),
                        None => 0.,
                    };

                    let fake_date = to_dbtime(Local::now() - Duration::days(1000));

                    let last_pattern_date = match last_pattern {
                        Some(val) => val.date,
                        None => fake_date,
                    };

                    let last_divergence_type = match last_divergence {
                        Some(val) => &val.divergence_type,
                        None => &DivergenceType::None,
                    };

                    let second_last_pattern_type = match len.cmp(&2) {
                        Ordering::Less => &PatternType::None,
                        Ordering::Greater => {
                            &instrument
                                .patterns
                                .local_patterns
                                .get(len - 2)
                                .unwrap()
                                .pattern_type
                        }
                        Ordering::Equal => {
                            &instrument
                                .patterns
                                .local_patterns
                                .get(len - 2)
                                .unwrap()
                                .pattern_type
                        }
                    };

                    let last_pattern_status =
                        get_pattern_status(last_pattern, second_last_pattern_type);
                    //let second_last_pattern_status = get_pattern_status(second_last_pattern);

                    if last_pattern_status != Status::Default {
                        let len = instrument.patterns.local_patterns.len();
                        instrument.patterns.local_patterns[len - 1].active.status =
                            last_pattern_status.clone();
                    }

                    // let extrema_pattern = instrument.patterns.extrema_patterns.last();
                    // let extrema_pattern_target = match extrema_pattern {
                    //     Some(val) => round(val.active.change, 0),
                    //     None => 0.,
                    // };
                    // let extrema_pattern_status = get_pattern_status(extrema_pattern);
                    // if extrema_pattern_status != Status::Default {
                    //     let len = instrument.patterns.extrema_patterns.len();
                    //     instrument.patterns.extrema_patterns[len - 1].active.status =
                    //         extrema_pattern_status.clone();
                    // }

                    let stoch_status = match stoch {
                        _x if stoch.current_a > stoch.current_b
                            && stoch.current_a > 20.
                            && stoch.current_a < 30. =>
                        {
                            Status::Bullish
                        }
                        _x if stoch.current_a < stoch.current_b => Status::Bearish,
                        _x if stoch.current_a > 70. => Status::Bearish,
                        _x if stoch.current_a > stoch.current_b && stoch.current_a > 40. => {
                            Status::Neutral
                        }
                        _ => Status::Neutral,
                    };

                    let macd_status = match macd {
                        _x if round(macd.current_a, 2) > round(macd.current_b, 2)
                            && macd.current_a > 0. =>
                        {
                            Status::Bullish
                        }
                        _x if round(macd.clone().current_a, 2)
                            < round(macd.clone().current_b, 2)
                            && round(macd.current_a, 2) < 0. =>
                        {
                            Status::Bearish
                        }
                        _x if round(macd.current_a, 1) >= round(macd.current_b, 1)
                            && round(macd.current_a, 1) <= 0. =>
                        {
                            Status::Neutral
                        }
                        //_x if macd.current_a < macd.current_b => Status::Bearish,
                        _ => Status::Neutral,
                    };

                    let rsi_status = match rsi {
                        _x if rsi.current_a < 30. => Status::Bullish,
                        _x if rsi.current_a > 60. => Status::Bearish,
                        _x if rsi.current_a > 40. && rsi.current_a < 60. => Status::Neutral,
                        _ => Status::Neutral,
                    };

                    let bb_status = Status::Neutral;

                    instrument.indicators.stoch.status = stoch_status.clone();
                    instrument.indicators.macd.status = macd_status.clone();
                    instrument.indicators.rsi.status = rsi_status.clone();
                    instrument.indicators.bb.status = bb_status.clone();

                    docs.push(instrument);
                }
                _ => {}
            }
        }
        docs
    }
}
