type Point = (usize, f64);
pub type DataPoints = Vec<Point>;
use crate::patterns::*;

use std::env;

#[derive(Debug, Clone)]
pub enum PatternType {
    TriangleSymmetricalTop,
    TriangleSymmetricalBottom,
    TriangleDescendantTop,
    TriangleDescendantBottom,
    TriangleAscendantTop,
    TriangleAscendantBottom,
    RectangleTop,
    RectangleBottom,
    ChannelUpTop,
    ChannelUpBottom,
    ChannelDownTop,
    ChannelDownBottom,
    BroadeningTop,
    BroadeningBottom,
    DoubleBottom,
    DoubleTop,
    None,
}

#[derive(Debug, Clone)]
pub enum PatternSize {
    Local,
    Extrema,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub pattern_size: PatternSize,
    pub data_points: DataPoints,
}

#[derive(Debug, Clone)]
pub struct Patterns {
    pub local_patterns: Vec<Pattern>,
    pub extrema_patterns: Vec<Pattern>,
}

impl Patterns {
    pub fn new() -> Self {
        Patterns {
            local_patterns: vec![],
            extrema_patterns: vec![],
        }
    }

    pub fn detect_pattern(
        &mut self,
        pattern_size: PatternSize,
        maxima: &Vec<(usize, f64)>,
        minima: &Vec<(usize, f64)>,
        current_price: &f64,
    ) {
        let local_max_points = env::var("PATTERNS_MAX_POINTS")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let min_points = env::var("PATTERNS_MIN_POINTS")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let window_size = env::var("PATTERNS_WINDOW_SIZE")
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let mut max_start = 0;
        let mut max_end = 0;
        let mut min_start = 0;
        let mut min_end = 0;
        let maxima_length = maxima.len();
        let minima_length = minima.len();
        if maxima_length >= min_points && minima_length >= min_points {
            if maxima_length > local_max_points {
                max_start = maxima_length - local_max_points;
                max_end = maxima_length;
            } else {
                max_start = 0;
                max_end = maxima_length;
            }

            if minima_length > local_max_points {
                min_start = minima_length - local_max_points;
                min_end = minima_length;
            } else {
                min_start = 0;
                min_end = minima_length;
            }

            let mut locals = [&maxima[max_start..max_end], &minima[min_start..min_end]].concat();

            locals.sort_by(|(id_a, _price_a), (id_b, _price_b)| id_a.cmp(id_b));
            locals.reverse();
            let mut iter = locals.windows(window_size);
            let mut no_pattern = true;
            while no_pattern {
                match iter.next() {
                    Some(window) => {
                        let data_points = window.to_vec();
                        if triangle::is_ascendant_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::TriangleAscendantTop,
                            );
                            //no_pattern = false;
                        } else if triangle::is_ascendant_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::TriangleAscendantBottom,
                            );
                            //no_pattern = false;
                        } else if triangle::is_descendant_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::TriangleDescendantTop,
                            );
                            // no_pattern = false;
                        } else if triangle::is_descendant_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::TriangleDescendantBottom,
                            );
                            // no_pattern = false;
                        } else if triangle::is_symmetrical_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::TriangleSymmetricalTop,
                            );
                            // no_pattern = false;
                        } else if triangle::is_symmetrical_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::TriangleSymmetricalBottom,
                            );
                            // no_pattern = false;
                        } else if rectangle::is_renctangle_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::RectangleTop,
                            );
                            // no_pattern = false;
                        } else if rectangle::is_renctangle_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::RectangleBottom,
                            );
                            //  no_pattern = false;
                        } else if channel::is_ascendant_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::ChannelUpTop,
                            );
                            //no_pattern = false;
                        } else if channel::is_ascendant_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::ChannelUpBottom,
                            );
                            // no_pattern = false;
                        } else if channel::is_descendant_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::ChannelDownTop,
                            );
                            //  no_pattern = false;
                        } else if channel::is_descendant_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::ChannelDownBottom,
                            );
                            // no_pattern = false;
                        } else if broadening::is_top(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::BroadeningTop,
                            );
                            // no_pattern = false;
                        } else if broadening::is_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::BroadeningBottom,
                            );
                            // no_pattern = false;
                        } else if double::is_top(&data_points, current_price) {
                            self.set_pattern(&data_points, &pattern_size, PatternType::DoubleTop);
                            // no_pattern = false;
                        } else if double::is_bottom(&data_points, current_price) {
                            self.set_pattern(
                                &data_points,
                                &pattern_size,
                                PatternType::DoubleBottom,
                            );
                            // no_pattern = false;
                        }
                    }
                    None => {
                        self.set_pattern(&vec![(0, 0.)], &pattern_size, PatternType::None);
                        no_pattern = false;
                    }
                }
            }
        } else {
            self.set_pattern(&vec![(0, 0.)], &pattern_size, PatternType::None);
        }
    }

    fn set_pattern(
        &mut self,
        data_points: &DataPoints,
        pattern_size: &PatternSize,
        pattern_type: PatternType,
    ) {
        match &pattern_size {
            PatternSize::Local => self.local_patterns.push(Pattern {
                pattern_type,
                pattern_size: pattern_size.clone(),
                data_points: data_points.to_owned(),
            }),
            PatternSize::Extrema => self.extrema_patterns.push(Pattern {
                pattern_type,
                pattern_size: pattern_size.clone(),
                data_points: data_points.to_owned(),
            }),
        };
    }

    // pub fn is_head_and_shoulders(
    //     &self,
    //     data: &DataPoints,
    //     _current_price: &f64,
    // ) -> Option<(Point, Point, Point)> {
    //     let hs_threshold = env::var("HEAD_AND_SHOULDERS_THRESHOLD")
    //         .unwrap()
    //         .parse::<f64>()
    //         .unwrap();
    //     if data[0] .1 > data[1] .1
    //         && data[2] .1 > data[0] .1
    //         && data[2] .1 > data[4] .1
    //         && (data[0] .1 - data[4] .1).abs()
    //             <= hs_threshold * comp::average(&[data[0] .1, data[4] .1])
    //         && (data[1] .1 - data[3] .1).abs()
    //             <= hs_threshold * comp::average(&[data[0] .1, data[4] .1])
    //     {
    //         println!("[HEAD AND SHOULDERS] {:?}", data);
    //         Some((data[0], data[1], data[2]))
    //     } else {
    //         None
    //     }
    // }

    // pub fn is_inverse_head_and_shoulders(
    //     &self,
    //     data: &DataPoints,
    //     _current_price: &f64,
    // ) -> Option<(Point, Point, Point)> {
    //     let hs_threshold = env::var("HEAD_AND_SHOULDERS_THRESHOLD")
    //         .unwrap()
    //         .parse::<f64>()
    //         .unwrap();
    //     if data[0] .1 < data[1] .1
    //         && data[2] .1 < data[0] .1
    //         && data[2] .1 < data[4] .1
    //         && (data[0] .1 - data[4] .1).abs()
    //             <= hs_threshold * comp::average(&[data[0] .1, data[4] .1])
    //         && (data[1] .1 - data[3] .1).abs()
    //             <= hs_threshold * comp::average(&[data[0] .1, data[4] .1])
    //     {
    //         println!("[INVERSE HEAD AND SHOULDERS] {:?}", data);
    //         Some((data[0], data[1], data[2]))
    //     } else {
    //         None
    //     }
    // }

    // pub fn upper_channel(&self) -> &UpperChannel {
    //     &self.upper_channel
    // }
    // pub fn is_upper_channel(&mut self, peaks: &Peaks) -> &UpperChannel {
    //     self.upper_channel.scan(peaks)
    // }
}

// impl<T> Default for Pattern<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }
