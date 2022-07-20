use super::strategy::*;

use crate::helpers::calc::*;
use crate::trade::*;
use async_trait::async_trait;
use rs_algo_shared::error::Result;
use rs_algo_shared::models::backtest_instrument::*;
use rs_algo_shared::models::backtest_strategy::*;
use rs_algo_shared::models::instrument::Instrument;

pub struct Ema<'a> {
    name: &'a str,
    strategy_type: StrategyType,
}

#[async_trait]
impl<'a> Strategy for Ema<'a> {
    fn new() -> Result<Self> {
        Ok(Self {
            name: "EMA_50_200",
            strategy_type: StrategyType::OnlyLong,
        })
    }

    fn name(&self) -> &str {
        self.name
    }

    fn strategy_type(&self) -> &StrategyType {
        &self.strategy_type
    }

    fn entry_long(&self, index: usize, instrument: &Instrument) -> bool {
        let prev_index = get_prev_index(index);

        let current_ema_50 = instrument.indicators.ema_b.data_a.get(index).unwrap();
        let current_ema_200 = instrument.indicators.ema_c.data_a.get(index).unwrap();

        let prev_ema_200 = instrument.indicators.ema_c.data_a.get(prev_index).unwrap();
        let prev_ema_50 = instrument.indicators.ema_b.data_a.get(prev_index).unwrap();

        let entry_condition = current_ema_50 > current_ema_200 && prev_ema_50 <= prev_ema_200;

        entry_condition
    }

    fn exit_long(&self, index: usize, instrument: &Instrument) -> bool {
        let prev_index = get_prev_index(index);

        let current_ema_50 = instrument.indicators.ema_b.data_a.get(index).unwrap();
        let current_ema_200 = instrument.indicators.ema_c.data_a.get(index).unwrap();

        let prev_ema_200 = instrument.indicators.ema_c.data_a.get(prev_index).unwrap();
        let prev_ema_50 = instrument.indicators.ema_b.data_a.get(prev_index).unwrap();

        let exit_condition = current_ema_50 < current_ema_200 && prev_ema_50 >= prev_ema_200;

        exit_condition
    }

    fn entry_short(&self, index: usize, instrument: &Instrument) -> bool {
        match self.strategy_type {
            StrategyType::LongShort => self.exit_long(index, instrument),
            StrategyType::OnlyShort => self.exit_long(index, instrument),
            _ => false,
        }
    }

    fn exit_short(&self, index: usize, instrument: &Instrument) -> bool {
        match self.strategy_type {
            StrategyType::LongShort => self.entry_long(index, instrument),
            StrategyType::OnlyShort => self.entry_long(index, instrument),
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
            instrument, trades_in, trades_out, self.name, equity, commission,
        )
    }
}
