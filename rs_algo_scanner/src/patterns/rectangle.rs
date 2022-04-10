use super::highs_lows::*;
use super::pattern::pattern_active_result;
use crate::candle::Candle;
use crate::prices::*;

use rs_algo_shared::helpers::comp::*;
use rs_algo_shared::models::*;
use std::env;

pub fn is_renctangle_top(data: &DataPoints) -> bool {
    let threshold = env::var("EQUAL_THRESHOLD").unwrap().parse::<f64>().unwrap();
    if upper_band_is_equal_top(data)
        && lower_band_is_equal_bottom(data)
        && is_equal_distance((data[0].1, data[1].1), (data[2].1, data[3].1), threshold)
    {
        true
    } else {
        false
    }
}

pub fn is_renctangle_bottom(data: &DataPoints) -> bool {
    let threshold = env::var("EQUAL_THRESHOLD").unwrap().parse::<f64>().unwrap();
    if upper_band_is_equal_bottom(data)
        && lower_band_is_equal_top(data)
        && is_equal_distance((data[0].1, data[1].1), (data[2].1, data[3].1), threshold)
    {
        true
    } else {
        false
    }
}

pub fn rectangle_top_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        &data,
        price_is_higher_upper_band_top(&data, candles, &pattern_type),
        price_is_lower_low_band_bottom(&data, candles, &pattern_type),
    )
}

pub fn rectangle_bottom_active(
    data: &DataPoints,
    candles: &Vec<Candle>,
    pattern_type: PatternType,
) -> PatternActive {
    pattern_active_result(
        &data,
        price_is_higher_upper_band_bottom(&data, candles, &pattern_type),
        price_is_lower_low_band_top(&data, candles, &pattern_type),
    )
}
