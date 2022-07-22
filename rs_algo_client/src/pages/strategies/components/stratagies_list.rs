use crate::routes::Route;
use crate::helpers::status::*;

use round::round;
use rs_algo_shared::models::backtest_strategy::BackTestStrategyResult;
use rs_algo_shared::helpers::status::*;
use rs_algo_shared::models::status::Status;
use rs_algo_shared::models::market::*;


use wasm_bindgen::prelude::*;
use yew::{function_component, html, Html, Properties};
use yew_router::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = get_query_value)]
    fn open_modal(modal: &str);
    #[wasm_bindgen(js_name = get_base_url)]
    fn get_base_url() -> String;
    fn close_modal(modal: &str);
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub strategies: Vec<BackTestStrategyResult>,
    pub market: Market,
}


#[function_component(StrategiesList)]
pub fn strategy_list(props: &Props) -> Html {
    let Props { strategies, market} = props;

    let strategy_list: Html = strategies
        .iter()
        .map(|strategy| {

            let profit_factor_status = get_profit_factor_status(strategy.avg_profit_factor);
            let profitable_trades_status = get_profitable_trades_status(strategy.avg_profitable_trades);
            let profit_status = get_profit_status(strategy.avg_net_profit_per, strategy.avg_buy_hold);
            let max_drawdown_status = get_max_drawdown_status(strategy.avg_max_drawdown);
            let avg_won_status = get_won_per_trade_status(strategy.avg_won_per_trade); 
            let avg_lost_status = get_lost_per_trade_status(strategy.avg_lost_per_trade);

            html! {
                <tr>
                    <td>
                    <Link<Route> to={Route::Strategy { market: strategy.market.to_string(), name: strategy.strategy.clone() }}>{ strategy.strategy.clone() }</Link<Route>>
                    </td>
                    <td>{ format!(" {:?} ", strategy.strategy_type )}</td>
                    <td class={get_status_class(&profit_status)}> { format!("{}%", round(strategy.avg_net_profit_per,2))}</td>
                    <td class={get_status_class(&profit_factor_status)}> { round(strategy.avg_profit_factor,2)}</td>
                    <td class={get_status_class(&profitable_trades_status)}> { format!("{}%", round(strategy.avg_profitable_trades,2))}</td>
                    <td class={get_status_class(&max_drawdown_status)}> { format!("{}%", round(strategy.avg_max_drawdown,2))}</td>
                    <td class={get_status_class(&avg_won_status)}>{ format!("{}%", round(strategy.avg_won_per_trade,2))}</td>
                    <td>{ strategy.avg_trades}</td>
                    <td>{ format!("{} / {}", strategy.avg_wining_trades, strategy.avg_losing_trades)} </td>
                    //<td class={get_status_class(&avg_lost_status)}>{ format!("{}%", round(strategy.avg_lost_per_trade,2))}</td>
                    <td>{ strategy.avg_stop_losses}</td>
                    <td>{ format!("{}%", round(strategy.avg_buy_hold,2))}</td>
                    <td> {format!("{}", strategy.date.to_chrono().format("%d/%m %H:%M"))}</td>
                </tr>
            }
        })
        .collect();

    let table = html! {
        <table class="table is-bordered">
            <thead class="has-background-grey-lighter">
                <tr>
                <th><abbr>{ "Strategy" }</abbr></th>
                <th><abbr>{ "Type" }</abbr></th>
                <th><abbr>{ "Net Profit" }</abbr></th>
                <th><abbr>{ "Profit F." }</abbr></th>
                <th><abbr>{ "Win Rate" }</abbr></th>
                <th><abbr>{ "Drawdown" }</abbr></th>
                <th><abbr>{ "Won p trade" }</abbr></th>
                <th><abbr>{ "Trades" }</abbr></th>
                <th><abbr>{ "Won / Lost" }</abbr></th>
                //<th><abbr>{ "Lost p trade" }</abbr></th>
                <th><abbr>{ "Stops " }</abbr></th>
                <th><abbr>{ "Buy & Hold" }</abbr></th>
                <th><abbr>{ "Updated" }</abbr></th>
                </tr>
            </thead>
            <tbody>
                { strategy_list }
            </tbody>
        </table>
    };

    table
}
