use super::strategy::*;
use crate::helpers::backtest::resolve_backtest;
use crate::helpers::calc::*;

use rs_algo_shared::error::Result;
use rs_algo_shared::indicators::Indicator;
use rs_algo_shared::models::backtest_instrument::*;
use rs_algo_shared::models::stop_loss::*;
use rs_algo_shared::models::strategy::StrategyType;
use rs_algo_shared::models::trade::{TradeIn, TradeOut};
use rs_algo_shared::scanner::instrument::*;

use async_trait::async_trait;

#[derive(Clone)]
pub struct MacdDual<'a> {
    name: &'a str,
    strategy_type: StrategyType,
    stop_loss: StopLoss,
}

#[async_trait]
impl<'a> Strategy for MacdDual<'a> {
    fn new() -> Result<Self> {
        let stop_loss = std::env::var("ATR_STOP_LOSS")
            .unwrap()
            .parse::<f64>()
            .unwrap();

        Ok(Self {
            stop_loss: init_stop_loss(StopLossType::Atr, stop_loss),
            name: "MacD_Dual",
            strategy_type: StrategyType::OnlyLongMTF,
        })
    }

    fn name(&self) -> &str {
        self.name
    }

    fn strategy_type(&self) -> &StrategyType {
        &self.strategy_type
    }

    fn update_stop_loss(&mut self, stop_type: StopLossType, price: f64) -> &StopLoss {
        self.stop_loss = update_stop_loss_values(&self.stop_loss, stop_type, price);
        &self.stop_loss
    }

    fn stop_loss(&self) -> &StopLoss {
        &self.stop_loss
    }

    fn entry_long(
        &mut self,
        index: usize,
        instrument: &Instrument,
        htf_instrument: &HTFInstrument,
    ) -> bool {
        let first_weekly_entry = get_htf_data(
            index,
            instrument,
            htf_instrument,
            |(idx, prev_idx, htf_inst)| {
                let curr_upper_macd_a = htf_inst.indicators.macd.get_data_a().get(idx).unwrap();
                let curr_upper_macd_b = htf_inst.indicators.macd.get_data_b().get(idx).unwrap();

                let prev_upper_macd_a =
                    htf_inst.indicators.macd.get_data_a().get(prev_idx).unwrap();
                let prev_upper_macd_b =
                    htf_inst.indicators.macd.get_data_b().get(prev_idx).unwrap();
                curr_upper_macd_a > curr_upper_macd_b && prev_upper_macd_b >= prev_upper_macd_a
            },
        );

        let upper_macd = get_htf_data(
            index,
            instrument,
            htf_instrument,
            |(idx, _prev_idx, htf_inst)| {
                let curr_upper_macd_a = htf_inst.indicators.macd.get_data_a().get(idx).unwrap();
                let curr_upper_macd_b = htf_inst.indicators.macd.get_data_b().get(idx).unwrap();
                curr_upper_macd_a > curr_upper_macd_b
            },
        );

        let prev_index = get_prev_index(index);

        let current_macd_a = instrument.indicators.macd.get_data_a().get(index).unwrap();
        let current_macd_b = instrument.indicators.macd.get_data_b().get(index).unwrap();

        let prev_macd_a = instrument
            .indicators
            .macd
            .get_data_a()
            .get(prev_index)
            .unwrap();
        let prev_macd_b = instrument
            .indicators
            .macd
            .get_data_a()
            .get(prev_index)
            .unwrap();

        first_weekly_entry
            || (upper_macd && current_macd_a > current_macd_b && prev_macd_b >= prev_macd_a)
    }

    fn exit_long(
        &mut self,
        index: usize,
        instrument: &Instrument,
        htf_instrument: &HTFInstrument,
    ) -> bool {
        let first_weekly_exit = get_htf_data(
            index,
            instrument,
            htf_instrument,
            |(idx, prev_idx, htf_inst)| {
                let curr_upper_macd_a = htf_inst.indicators.macd.get_data_a().get(idx).unwrap();
                let curr_upper_macd_b = htf_inst.indicators.macd.get_data_b().get(idx).unwrap();

                let prev_upper_macd_a =
                    htf_inst.indicators.macd.get_data_a().get(prev_idx).unwrap();
                let prev_upper_macd_b =
                    htf_inst.indicators.macd.get_data_b().get(prev_idx).unwrap();
                curr_upper_macd_a < curr_upper_macd_b && prev_upper_macd_a >= prev_upper_macd_b
            },
        );

        let prev_index = get_prev_index(index);

        let current_macd_a = instrument.indicators.macd.get_data_a().get(index).unwrap();
        let current_macd_b = instrument.indicators.macd.get_data_b().get(index).unwrap();
        let low_price = &instrument.data.get(index).unwrap().low;
        let prev_macd_a = instrument
            .indicators
            .macd
            .get_data_a()
            .get(prev_index)
            .unwrap();
        let prev_macd_b = instrument
            .indicators
            .macd
            .get_data_a()
            .get(prev_index)
            .unwrap();

        let exit_condition =
            first_weekly_exit || current_macd_a < current_macd_b && prev_macd_b <= prev_macd_a;

        if exit_condition {
            self.update_stop_loss(StopLossType::Trailing, *low_price);
        }

        exit_condition
    }

    fn entry_short(
        &mut self,
        index: usize,
        instrument: &Instrument,
        htf_instrument: &HTFInstrument,
    ) -> bool {
        match self.strategy_type {
            StrategyType::LongShort => self.exit_long(index, instrument, htf_instrument),
            StrategyType::LongShortMTF => self.exit_long(index, instrument, htf_instrument),
            StrategyType::OnlyShort => self.exit_long(index, instrument, htf_instrument),
            _ => false,
        }
    }

    fn exit_short(
        &mut self,
        index: usize,
        instrument: &Instrument,
        htf_instrument: &HTFInstrument,
    ) -> bool {
        match self.strategy_type {
            StrategyType::LongShort => self.entry_long(index, instrument, htf_instrument),
            StrategyType::LongShortMTF => self.entry_long(index, instrument, htf_instrument),
            StrategyType::OnlyShort => self.entry_long(index, instrument, htf_instrument),
            _ => false,
        }
    }

    fn backtest_result(
        &self,
        instrument: &Instrument,
        trades_in: Vec<TradeIn>,
        trades_out: Vec<TradeOut>,
        equity: f64,
        commission: f64,
    ) -> BackTestResult {
        resolve_backtest(
            instrument,
            &self.strategy_type,
            trades_in,
            trades_out,
            self.name,
            equity,
            commission,
        )
    }
}
