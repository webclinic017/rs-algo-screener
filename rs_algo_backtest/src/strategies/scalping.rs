use super::strategy::*;
use crate::helpers::backtest::resolve_backtest;

use rs_algo_shared::error::Result;
use rs_algo_shared::helpers::calc;
use rs_algo_shared::indicators::Indicator;
use rs_algo_shared::models::backtest_instrument::*;
use rs_algo_shared::models::order::{Order, OrderType};
use rs_algo_shared::models::stop_loss::*;
use rs_algo_shared::models::strategy::StrategyType;
use rs_algo_shared::models::trade::{Operation, TradeIn, TradeOut};
use rs_algo_shared::scanner::instrument::*;

use async_trait::async_trait;

#[derive(Clone)]
pub struct Scalping<'a> {
    name: &'a str,
    strategy_type: StrategyType,
    //stop_loss: StopLoss,
}

impl<'a> Strategy for Scalping<'a> {
    fn new() -> Result<Self> {
        let stop_loss = std::env::var("BACKTEST_ATR_STOP_LOSS")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        Ok(Self {
            //stop_loss: init_stop_loss(StopLossType::Atr, stop_loss),
            name: "Scalping",
            strategy_type: StrategyType::OnlyLongMultiTF,
        })
    }

    fn name(&self) -> &str {
        self.name
    }

    fn strategy_type(&self) -> &StrategyType {
        &self.strategy_type
    }

    // fn update_stop_loss(&mut self, stop_type: StopLossType, price: f64) -> &StopLoss {
    //     // self.stop_loss = update_stop_loss_values(&self.stop_loss, stop_type, price);
    //     &self.stop_loss
    // }

    // fn stop_loss(&self) -> &StopLoss {
    //     &self.stop_loss
    // }

    // fn create_stop_loss() {
    //     create_stop_loss(&entry_type, instrument, nex_candle_index, stop_loss)
    // }

    fn entry_long(
        &mut self,
        index: usize,
        instrument: &Instrument,
        upper_tf_instrument: &HigherTMInstrument,
        spread: f64,
    ) -> Operation {
        let close_price = &instrument.data.get(index).unwrap().close();

        // let first_htf_emas = calc::get_upper_timeframe_data(
        //     index,
        //     instrument,
        //     upper_tf_instrument,
        //     |(idx, prev_idx, upper_inst)| {
        //         let htf_ema_8 = upper_inst.indicators.ema_a.get_data_a().get(idx).unwrap();
        //         let prev_htf_ema_8 = upper_inst
        //             .indicators
        //             .ema_a
        //             .get_data_a()
        //             .get(prev_idx)
        //             .unwrap();
        //         let htf_ema_21 = upper_inst.indicators.ema_c.get_data_a().get(idx).unwrap();
        //         let prev_htf_ema_21 = upper_inst
        //             .indicators
        //             .ema_c
        //             .get_data_a()
        //             .get(prev_idx)
        //             .unwrap();
        //         htf_ema_8 > htf_ema_21 && prev_htf_ema_8 <= prev_htf_ema_21
        //     },
        // );
        let htf_emas = calc::get_upper_timeframe_data(
            index,
            instrument,
            upper_tf_instrument,
            |(idx, prev_idx, upper_inst)| {
                let htf_ema_8 = upper_inst.indicators.ema_a.get_data_a().get(idx).unwrap();
                let prev_htf_ema_8 = upper_inst
                    .indicators
                    .ema_a
                    .get_data_a()
                    .get(prev_idx)
                    .unwrap();
                let htf_ema_21 = upper_inst.indicators.ema_c.get_data_a().get(idx).unwrap();
                let prev_htf_ema_21 = upper_inst
                    .indicators
                    .ema_c
                    .get_data_a()
                    .get(prev_idx)
                    .unwrap();
                htf_ema_8 > htf_ema_21 && close_price > htf_ema_21
            },
        );

        let prev_index = calc::get_prev_index(index);
        let next_index = index + 1;
        let data = &instrument.data();
        let close_price = &data.get(index).unwrap().close();
        let prev_close_price = &data.get(prev_index).unwrap().close();
        let open_price = data.get(next_index).unwrap().close();
        let ema_8 = instrument.indicators.ema_a.get_data_a().get(index).unwrap();
        let prev_ema_8 = instrument
            .indicators
            .ema_a
            .get_data_a()
            .get(prev_index)
            .unwrap();
        let ema_13 = instrument.indicators.ema_b.get_data_a().get(index).unwrap();
        let ema_21 = instrument.indicators.ema_c.get_data_a().get(index).unwrap();

        let entry_condition = htf_emas
            && (close_price < ema_8
                && prev_close_price >= prev_ema_8
                && ema_8 > ema_13
                && ema_13 > ema_21);

        let trigger_price = match self.strategy_type().is_long_only() {
            true => {
                data[index - 5..index]
                    .iter()
                    .max_by(|x, y| x.high().partial_cmp(&y.high()).unwrap())
                    .map(|x| x.close())
                    .unwrap()
                    + calc::to_pips(&3.)
            }
            false => {
                data[index - 5..index]
                    .iter()
                    .min_by(|x, y| x.high().partial_cmp(&y.high()).unwrap())
                    .map(|x| x.close())
                    .unwrap()
                    - calc::to_pips(&3.)
            }
        };

        let buy_price = match self.strategy_type().is_long_only() {
            true => trigger_price + spread,
            false => trigger_price - spread,
        };

        let pips_margin = 3.;

        let risk = match self.strategy_type().is_long_only() {
            true => buy_price - close_price - calc::to_pips(&pips_margin),
            false => buy_price + close_price + calc::to_pips(&pips_margin),
        };

        let target_price = match self.strategy_type().is_long_only() {
            true => buy_price + risk * 1.,
            false => buy_price - risk * 1.,
        };

        match entry_condition {
            //true => Operation::MarketIn(Some(vec![OrderType::StopLoss(StopLossType::Atr)])),
            true => Operation::Order(vec![
                OrderType::BuyOrder(open_price, trigger_price),
                OrderType::SellOrder(open_price, target_price),
                OrderType::StopLoss(StopLossType::Pips(pips_margin)),
            ]),
            false => Operation::None,
        }
    }

    fn exit_long(
        &mut self,
        index: usize,
        instrument: &Instrument,
        upper_tf_instrument: &HigherTMInstrument,
        spread: f64,
    ) -> Operation {
        let close_price = &instrument.data.get(index).unwrap().close();

        let htf_emas = calc::get_upper_timeframe_data(
            index,
            instrument,
            upper_tf_instrument,
            |(idx, prev_idx, upper_inst)| {
                let htf_ema_8 = upper_inst.indicators.ema_a.get_data_a().get(idx).unwrap();
                let prev_htf_ema_8 = upper_inst
                    .indicators
                    .ema_a
                    .get_data_a()
                    .get(prev_idx)
                    .unwrap();
                let htf_ema_21 = upper_inst.indicators.ema_c.get_data_a().get(idx).unwrap();
                let prev_htf_ema_21 = upper_inst
                    .indicators
                    .ema_c
                    .get_data_a()
                    .get(prev_idx)
                    .unwrap();
                htf_ema_8 < htf_ema_21 && close_price < htf_ema_21
            },
        );

        let prev_index = calc::get_prev_index(index);
        let next_index = index + 1;
        let data = &instrument.data();
        let close_price = &data.get(index).unwrap().close();
        let prev_close_price = &data.get(prev_index).unwrap().close();
        let open_price = data.get(next_index).unwrap().close();
        let ema_8 = instrument.indicators.ema_a.get_data_a().get(index).unwrap();
        let prev_ema_8 = instrument
            .indicators
            .ema_a
            .get_data_a()
            .get(prev_index)
            .unwrap();
        let ema_13 = instrument.indicators.ema_b.get_data_a().get(index).unwrap();
        let ema_21 = instrument.indicators.ema_c.get_data_a().get(index).unwrap();

        let exit_condition = htf_emas
            || (ema_8 < ema_13 || ema_13 < ema_21 || ema_8 < ema_21)
            || (close_price > ema_8
                && prev_close_price <= prev_ema_8
                && ema_8 < ema_13
                && ema_13 < ema_21);

        let trigger_price = match self.strategy_type().is_long_only() {
            true => {
                data[index - 5..index]
                    .iter()
                    .max_by(|x, y| x.high().partial_cmp(&y.high()).unwrap())
                    .map(|x| x.close())
                    .unwrap()
                    + calc::to_pips(&3.)
            }
            false => {
                data[index - 5..index]
                    .iter()
                    .min_by(|x, y| x.high().partial_cmp(&y.high()).unwrap())
                    .map(|x| x.close())
                    .unwrap()
                    - calc::to_pips(&3.)
            }
        };

        let buy_price = match self.strategy_type().is_long_only() {
            true => trigger_price + spread,
            false => trigger_price - spread,
        };

        let risk = match self.strategy_type().is_long_only() {
            true => buy_price - close_price - calc::to_pips(&3.),
            false => buy_price + close_price + calc::to_pips(&3.),
        };

        let target_price = match self.strategy_type().is_long_only() {
            true => buy_price + risk * 1.,
            false => buy_price - risk * 1.,
        };

        match exit_condition {
            //true => Operation::MarketIn(Some(vec![OrderType::StopLoss(StopLossType::Atr)])),
            // true => Operation::Order(vec![
            //     OrderType::BuyOrder(open_price, trigger_price),
            //     OrderType::SellOrder(open_price, target_price),
            //     OrderType::StopLoss(StopLossType::Pips(3.)),
            // ]),
            true => Operation::None,
            false => Operation::None,
        }
    }

    fn entry_short(
        &mut self,
        index: usize,
        instrument: &Instrument,
        upper_tf_instrument: &HigherTMInstrument,
        spread: f64,
    ) -> Operation {
        match self.strategy_type {
            StrategyType::LongShort | StrategyType::LongShortMultiTF | StrategyType::OnlyShort => {
                self.exit_long(index, instrument, upper_tf_instrument, spread)
            }
            _ => Operation::None,
        }
    }

    fn exit_short(
        &mut self,
        index: usize,
        instrument: &Instrument,
        upper_tf_instrument: &HigherTMInstrument,
        spread: f64,
    ) -> Operation {
        match self.strategy_type {
            StrategyType::LongShort | StrategyType::LongShortMultiTF | StrategyType::OnlyShort => {
                self.entry_long(index, instrument, upper_tf_instrument, spread)
            }
            _ => Operation::None,
        }
    }

    fn backtest_result(
        &self,
        instrument: &Instrument,
        trades_in: Vec<TradeIn>,
        trades_out: Vec<TradeOut>,
        orders: Vec<Order>,
        equity: f64,
        commission: f64,
    ) -> BackTestResult {
        resolve_backtest(
            instrument,
            &self.strategy_type,
            trades_in,
            trades_out,
            orders,
            self.name,
            equity,
            commission,
        )
    }
}
