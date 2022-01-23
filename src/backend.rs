use crate::candle::CandleType;
use crate::error::Result;
use crate::indicators::macd::Macd;
use crate::indicators::stoch::Stoch;
use crate::indicators::Indicator;
use crate::instrument::Instrument;
use crate::patterns::horizontal_levels::HorizontalLevelType;

use plotters::prelude::*;
use std::env;
use ta::indicators::{ExponentialMovingAverage, SimpleMovingAverage};
use ta::Next;

pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, instrument: &Instrument) -> Result<()> {
        let to_date = instrument.data().last().unwrap().date();
        let from_date = instrument.data().first().unwrap().date();
        let peaks_marker_distance = env::var("MARKERS_DISTANCE")
            .unwrap()
            .parse::<f64>()
            .unwrap();
        let output_file = [
            &env::var("OUTPUT_FOLDER").unwrap(),
            instrument.ticker(),
            ".png",
        ]
        .concat();
        let min_price = instrument.min_price();
        let max_price = instrument.max_price();

        let data = instrument.data();
        let local_maxima = instrument.peaks().local_maxima();
        let smooth_highs = instrument.peaks().smooth_highs();
        let smooth_lows = instrument.peaks().smooth_lows();
        let local_minima = instrument.peaks().local_minima();
        let extrema_maxima = instrument.peaks().extrema_maxima();
        let extrema_minima = instrument.peaks().extrema_minima();
        let horizontal_levels = instrument.horizontal_levels().horizontal_levels();
        //let upper_channel = instrument.patterns().upper_channel();

        let rsi = instrument.indicators().rsi();
        let rsi_a = rsi.data_a();

        let stoch = instrument.indicators().stoch();
        let stoch_a = stoch.data_a();
        let stoch_b = stoch.data_b();

        let macd = instrument.indicators().macd();
        let macd_a = macd.data_a();
        let macd_b = macd.data_b();

        let root = BitMapBackend::new(&output_file, (1024, 768)).into_drawing_area();
        let (upper, lower) = root.split_vertically((70).percent());
        let (lower_2, lower_1) = lower.split_vertically((70).percent());
        let (indicator_1, indicator_2) = lower_2.split_vertically((50).percent());

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&upper)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .margin(5)
            .caption(instrument.ticker(), ("sans-serif", 14.0).into_font())
            .build_cartesian_2d(from_date..to_date, min_price..max_price)
            .unwrap();

        chart
            .configure_mesh()
            .light_line_style(&WHITE)
            .draw()
            .unwrap();
        chart
            .draw_series(data.iter().map(|candle| {
                let candle_color: (ShapeStyle, ShapeStyle) = match candle {
                    _x if candle.close() >= candle.open() => (GREEN.filled(), GREEN.filled()),
                    _x if candle.close() <= candle.open() => (RED.filled(), RED.filled()),
                    _ => (GREEN.filled(), GREEN.filled()),
                };

                let (bullish, bearish) = match candle.candle_type() {
                    CandleType::Engulfing => (BLUE.filled(), BLUE.filled()),
                    _ => candle_color,
                };

                CandleStick::new(
                    candle.date(),
                    candle.open(),
                    candle.high(),
                    candle.low(),
                    candle.close(),
                    bullish,
                    bearish,
                    2,
                )
            }))
            .unwrap();

        // UPPER CHANNEL
        // for x in local_maxima.iter() {
        //     chart
        //         .draw_series(LineSeries::new(
        //             (0..).zip(local_maxima.iter()).map(|(_k, highs)| {
        //                 let idx = highs.0;
        //                 let value = highs.1;
        //                 let date = data[idx].date();
        //                 (date, value)
        //             }),
        //             &BLUE,
        //         ))
        //         .unwrap();
        // }

        for x in smooth_highs.iter() {
            chart
                .draw_series(LineSeries::new(
                    (0..).zip(smooth_highs.iter()).map(|(_k, highs)| {
                        let idx = highs.0;
                        let value = highs.1;
                        let date = data[idx].date();
                        (date, value)
                    }),
                    &BLUE,
                ))
                .unwrap();
        }

        chart
            .draw_series(data.iter().enumerate().map(|(i, candle)| {
                if local_maxima.contains(&(i, candle.close())) {
                    return TriangleMarker::new(
                        (
                            candle.date(),
                            candle.high() + candle.close() / peaks_marker_distance + 5.,
                        ),
                        -4,
                        BLUE.filled(),
                    );
                } else {
                    return TriangleMarker::new((candle.date(), candle.close()), 0, &TRANSPARENT);
                }
            }))
            .unwrap();

        chart
            .draw_series(data.iter().enumerate().map(|(i, candle)| {
                if local_minima.contains(&(i, candle.close())) {
                    return TriangleMarker::new(
                        (
                            candle.date(),
                            candle.high() + candle.high() / peaks_marker_distance - 10.,
                        ),
                        4,
                        RED.filled(),
                    );
                } else {
                    return TriangleMarker::new((candle.date(), candle.high()), 0, &TRANSPARENT);
                }
            }))
            .unwrap();

        // for x in smooth_lows.iter() {
        //     chart
        //         .draw_series(LineSeries::new(
        //             (0..).zip(smooth_lows.iter()).map(|(_k, highs)| {
        //                 let idx = highs.0;
        //                 let value = highs.1;
        //                 let date = data[idx].date();
        //                 (date, value)
        //             }),
        //             &BLUE,
        //         ))
        //         .unwrap();
        // }

        // for x in local_minima.iter() {
        //     chart
        //         .draw_series(LineSeries::new(
        //             (0..).zip(smooth_highs.iter()).map(|(_k, highs)| {
        //                 let idx = highs.0;
        //                 let value = highs.1;
        //                 let date = data[idx].date();
        //                 (date, value)
        //             }),
        //             &BLUE,
        //         ))
        //         .unwrap();
        // }
        // chart
        //   .draw_series(LineSeries::new(
        //     (0..).zip(upper_channel.lower_band()[2].iter()).map(|(_k, highs)| {
        //       let idx = highs.0;
        //       let value = highs.1;
        //       let date = data[idx].date();
        //       (date, value)
        //     }),
        //     &RED,
        //   ))
        //   .unwrap();

        // chart
        //   .draw_series(LineSeries::new(
        //     (0..)
        //       .zip(upper_channel.lower_band()[0].iter())
        //       .map(|(key, value)| {
        //         let date = data[value.0].date();
        //         (date, value.1)
        //       }),
        //     &RED,
        //   ))
        //   .unwrap();
        // // }

        // chart
        //   .draw_series(LineSeries::new(
        //     (0..)
        //       .zip(upper_channel.lower_band().iter())
        //       .map(|(key, value)| {
        //         let date = data[value.0].date();
        //         //println!("222222, {:?}, {:?}", date, value);
        //         (date, value.1)
        //       }),
        //     &RED,
        //   ))
        //   .unwrap();

        // HORIZONTAL LEVELS
        /*
        for x in horizontal_levels.iter() {
            let color = match x.1.level_type() {
                HorizontalLevelType::Support => BLUE.filled(),
                HorizontalLevelType::Resistance => RED.filled(),
                _ => TRANSPARENT.filled(),
            };
            chart
                .draw_series(LineSeries::new(
                    (0..)
                        .zip(data.iter())
                        .map(|(_id, candle)| (candle.date(), *x.1.price())),
                    color,
                ))
                .unwrap();
        }

        let mut ema20 = ExponentialMovingAverage::new(20).unwrap();
        let mut ema50 = ExponentialMovingAverage::new(50).unwrap();
        let mut ema200 = ExponentialMovingAverage::new(200).unwrap();
        chart
          .draw_series(LineSeries::new(
            (0..).zip(data.iter()).map(|(_id, candle)| {
              let date = candle.date();
              let value = ema20.next(candle.close());
              (date, value)
            }),
            &BLUE,
          ))
          .unwrap();

        chart
          .draw_series(LineSeries::new(
            (0..).zip(data.iter()).map(|(_id, candle)| {
              let date = candle.date();
              let value = ema50.next(candle.close());
              (date, value)
            }),
            &RED,
          ))
          .unwrap();

        chart
          .draw_series(LineSeries::new(
            (0..).zip(data.iter()).map(|(_id, candle)| {
              let date = candle.date();
              let value = ema200.next(candle.close());
              (date, value)
            }),
            &YELLOW,
          ))
          .unwrap();

        // PEAKS

        chart
          .draw_series(data.iter().enumerate().map(|(i, candle)| {
            if local_maxima.contains(&(i, candle.high())) {
              return TriangleMarker::new(
                (candle.date(), candle.high() + candle.high() / peaks_marker_distance + 5.),
                -4,
                BLUE.filled(),
              );
            } else {
              return TriangleMarker::new((candle.date(), candle.high()), 0, &TRANSPARENT);
            }
          }))
          .unwrap();

        chart
          .draw_series(data.iter().enumerate().map(|(i, candle)| {
            if local_minima.contains(&(i, candle.low())) {
              return TriangleMarker::new(
                (candle.date(), candle.low() - candle.low() / peaks_marker_distance - 5.),
                4,
                BLUE.filled(),
              );
            } else {
              return TriangleMarker::new((candle.date(), candle.low()), 0, &TRANSPARENT);
            }
          }))
          .unwrap();

        chart
          .draw_series(data.iter().enumerate().map(|(i, candle)| {
            if extrema_maxima.contains(&(i, candle.high())) {
              return TriangleMarker::new(
                (candle.date(), candle.low() - candle.low() / peaks_marker_distance + 25.),
                -4,
                RED.filled(),
              );
            } else {
              return TriangleMarker::new((candle.date(), candle.low()), 0, &TRANSPARENT);
            }
          }))
          .unwrap();

        chart
          .draw_series(data.iter().enumerate().map(|(i, candle)| {
            if extrema_minima.contains(&(i, candle.low())) {
              return TriangleMarker::new(
                (candle.date(), candle.low() - candle.low() / peaks_marker_distance - 25.),
                4,
                RED.filled(),
              );
            } else {
              return TriangleMarker::new((candle.date(), candle.low()), 0, &TRANSPARENT);
            }
          }))
          .unwrap();

          */

        let mut rsi_pannel = ChartBuilder::on(&lower_1)
            .x_label_area_size(40)
            .y_label_area_size(40)
            //.margin(2)
            //.caption("RSI", ("sans-serif", 8.0).into_font())
            .build_cartesian_2d(from_date..to_date, -0f64..100f64)
            .unwrap();
        //rsi_pannel.configure_mesh().light_line_style(&WHITE).draw().unwrap();
        rsi_pannel
            .draw_series(LineSeries::new(
                (0..)
                    .zip(data.iter())
                    .map(|(id, candle)| (candle.date(), rsi_a[id])),
                &RED,
            ))
            .unwrap();

        let mut stoch_pannel = ChartBuilder::on(&indicator_1)
            .x_label_area_size(40)
            .y_label_area_size(40)
            // .margin(2)
            //.caption("MACD", ("sans-serif", 8.0).into_font())
            .build_cartesian_2d(from_date..to_date, -0f64..100f64)
            .unwrap();
        //stoch_pannel.configure_mesh().light_line_style(&WHITE).draw().unwrap();
        stoch_pannel
            .draw_series(LineSeries::new(
                (0..)
                    .zip(data.iter())
                    .map(|(id, candle)| (candle.date(), stoch_a[id])),
                &BLUE,
            ))
            .unwrap();

        stoch_pannel
            .draw_series(LineSeries::new(
                (0..)
                    .zip(data.iter())
                    .map(|(id, candle)| (candle.date(), stoch_b[id])),
                &RED,
            ))
            .unwrap();

        let mut stoch_pannel = ChartBuilder::on(&indicator_2)
            .x_label_area_size(40)
            .y_label_area_size(40)
            //.margin(2)
            //.caption("STOCH", ("sans-serif", 8.0).into_font())
            .build_cartesian_2d(from_date..to_date, -10f64..10f64)
            .unwrap();
        // stoch_pannel.configure_mesh().light_line_style(&WHITE).draw().unwrap();
        stoch_pannel
            .draw_series(LineSeries::new(
                (0..)
                    .zip(data.iter())
                    .map(|(id, candle)| (candle.date(), macd_a[id])),
                &BLUE,
            ))
            .unwrap();

        stoch_pannel
            .draw_series(LineSeries::new(
                (0..)
                    .zip(data.iter())
                    .map(|(id, candle)| (candle.date(), macd_b[id])),
                &RED,
            ))
            .unwrap();

        root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
        println!("[Backend] Ok file saved in {}", output_file);
        Ok(())
    }
}
