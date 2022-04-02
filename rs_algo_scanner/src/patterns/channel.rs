use super::highs_lows::*;
use super::pattern::pattern_active_result;
use crate::candle::Candle;
use crate::prices::*;
use rs_algo_shared::helpers::comp::*;

use rs_algo_shared::models::*;

pub fn is_ascendant_top(data: &DataPoints) -> bool {
    if is_higher_highs_top(data)
        && is_higher_lows_top(data)
        && is_equal_distance((data[0].1, data[1].1), (data[2].1, data[3].1))
    {
        true
    } else {
        false
    }
}

pub fn is_ascendant_bottom(data: &DataPoints) -> bool {
    if is_higher_highs_bottom(data)
        && is_higher_lows_bottom(data)
        && is_equal_distance((data[0].1, data[1].1), (data[2].1, data[3].1))
    {
        true
    } else {
        false
    }
}

pub fn is_descendant_top(data: &DataPoints) -> bool {
    if is_lower_highs_top(data)
        && is_lower_lows_top(data)
        && is_equal_distance((data[0].1, data[1].1), (data[2].1, data[3].1))
    {
        true
    } else {
        false
    }
}

pub fn is_descendant_bottom(data: &DataPoints) -> bool {
    if is_lower_highs_bottom(data)
        && is_lower_lows_bottom(data)
        && is_equal_distance((data[0].1, data[1].1), (data[2].1, data[3].1))
    {
        true
    } else {
        false
    }
}

pub fn channel_top_active(data: &DataPoints, candles: &Vec<Candle>) -> PatternActive {
    pattern_active_result(
        &data,
        price_is_higher_upper_band_top(&data, candles),
        price_is_lower_low_band_top(&data, candles),
    )
}

pub fn channel_bottom_active(data: &DataPoints, candles: &Vec<Candle>) -> PatternActive {
    pattern_active_result(
        &data,
        price_is_higher_upper_band_bottom(&data, candles),
        price_is_lower_low_band_bottom(&data, candles),
    )
}
