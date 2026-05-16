// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
};

use nautilus_model::{
    data::{Bar, QuoteTick, TradeTick},
    enums::PriceType,
};

use crate::{
    average::{MovingAverageFactory, MovingAverageType},
    indicator::{Indicator, MovingAverage},
};

/// Calculates the slope (rate of change) of a moving average over a specified
/// lookback range.
///
/// The MASlope indicator computes a moving average of the input prices, stores
/// the MA history, and calculates the slope between two points in that history.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.indicators", unsendable)
)]
pub struct MASlope {
    /// The period for the moving average (> 0).
    pub ma_period: usize,
    /// The starting point for slope calculation (bars ago).
    pub start_bars_ago: usize,
    /// The ending point for slope calculation (bars ago).
    pub end_bars_ago: usize,
    /// The moving average type for calculations.
    pub ma_type: MovingAverageType,
    /// The price type used for calculations.
    pub price_type: PriceType,
    /// The last indicator value (slope).
    pub value: f64,
    pub initialized: bool,
    has_inputs: bool,
    ma: Box<dyn MovingAverage + Send + 'static>,
    ma_prices: VecDeque<f64>,
}

impl Display for MASlope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({},{},{},{})",
            self.name(),
            self.ma_period,
            self.start_bars_ago,
            self.end_bars_ago,
            self.ma_type
        )
    }
}

impl Indicator for MASlope {
    fn name(&self) -> String {
        stringify!(MASlope).to_string()
    }

    fn has_inputs(&self) -> bool {
        self.has_inputs
    }

    fn initialized(&self) -> bool {
        self.initialized
    }

    fn handle_quote(&mut self, quote: &QuoteTick) {
        self.update_raw(quote.extract_price(self.price_type).into());
    }

    fn handle_trade(&mut self, trade: &TradeTick) {
        self.update_raw((&trade.price).into());
    }

    fn handle_bar(&mut self, bar: &Bar) {
        self.update_raw((&bar.close).into());
    }

    fn reset(&mut self) {
        self.value = 0.0;
        self.ma.reset();
        self.ma_prices.clear();
        self.has_inputs = false;
        self.initialized = false;
    }
}

impl MASlope {
    /// Creates a new [`MASlope`] instance.
    ///
    /// # Parameters
    ///
    /// - `ma_period`: The period for the moving average (> 0).
    /// - `start_bars_ago`: Starting point for slope calculation.
    ///   If negative, interpreted as `ma_period + start_bars_ago`.
    /// - `end_bars_ago`: Ending point for slope calculation (default: 0).
    ///   If negative, interpreted as `ma_period + end_bars_ago`.
    /// - `ma_type`: The moving average type (default: SIMPLE).
    /// - `price_type`: The price type for extracting values (default: LAST).
    ///
    /// # Panics
    ///
    /// Panics if `ma_period` is not positive (> 0).
    /// Panics if `start_bars_ago` is not greater than `end_bars_ago`.
    #[must_use]
    pub fn new(
        ma_period: usize,
        start_bars_ago: isize,
        end_bars_ago: Option<isize>,
        ma_type: Option<MovingAverageType>,
        price_type: Option<PriceType>,
    ) -> Self {
        assert!(
            ma_period > 0,
            "MASlope::new → `ma_period` must be positive (> 0); got {ma_period}"
        );

        // Normalize negative indices
        let start_bars_ago_normalized = if start_bars_ago >= 0 {
            start_bars_ago as usize
        } else {
            (ma_period as isize + start_bars_ago) as usize
        };

        let end_bars_ago_value = end_bars_ago.unwrap_or(0);
        let end_bars_ago_normalized = if end_bars_ago_value >= 0 {
            end_bars_ago_value as usize
        } else {
            (ma_period as isize + end_bars_ago_value) as usize
        };

        assert!(
            start_bars_ago_normalized > end_bars_ago_normalized,
            "MASlope::new → `start_bars_ago` must be > `end_bars_ago`; got start={start_bars_ago_normalized}, end={end_bars_ago_normalized}"
        );

        let ma_type_value = ma_type.unwrap_or(MovingAverageType::Simple);

        Self {
            ma_period,
            start_bars_ago: start_bars_ago_normalized,
            end_bars_ago: end_bars_ago_normalized,
            ma_type: ma_type_value,
            price_type: price_type.unwrap_or(PriceType::Last),
            value: 0.0,
            initialized: false,
            has_inputs: false,
            ma: MovingAverageFactory::create(ma_type_value, ma_period),
            ma_prices: VecDeque::with_capacity(ma_period),
        }
    }

    /// Clamps an index to the valid range [0, n-1].
    fn clamp_index(&self, idx: usize, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        idx.min(n - 1)
    }
}

impl MovingAverage for MASlope {
    fn value(&self) -> f64 {
        self.value
    }

    fn count(&self) -> usize {
        self.ma.count()
    }

    fn update_raw(&mut self, close: f64) {
        // Update the internal moving average
        self.ma.update_raw(close);

        // Store MA value at the front (newest first)
        self.ma_prices.push_front(self.ma.value());

        // Maintain max capacity
        if self.ma_prices.len() > self.ma_period {
            self.ma_prices.pop_back();
        }

        // Calculate slope
        let n = self.ma_prices.len();
        let start_idx = self.clamp_index(self.start_bars_ago, n);
        let end_idx = self.clamp_index(self.end_bars_ago, n);

        if start_idx == end_idx {
            self.value = 0.0;
        } else {
            let ma_start = self.ma_prices[start_idx];
            let ma_end = self.ma_prices[end_idx];
            self.value = (ma_end - ma_start) / (start_idx as f64 - end_idx as f64);
        }

        // Initialization logic
        if !self.initialized {
            self.has_inputs = true;
            if self.ma.initialized() {
                self.initialized = true;
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use nautilus_model::{
        data::{Bar, QuoteTick, TradeTick},
        enums::PriceType,
    };
    use rstest::rstest;

    use crate::{
        average::{MovingAverageType, ma_slope::MASlope},
        indicator::{Indicator, MovingAverage},
        stubs::*,
    };

    #[rstest]
    fn test_ma_slope_initialized() {
        let ma_slope = MASlope::new(10, 5, Some(0), Some(MovingAverageType::Simple), None);
        let display_str = format!("{ma_slope}");
        assert_eq!(display_str, "MASlope(10,5,0,SIMPLE)");
        assert_eq!(ma_slope.ma_period, 10);
        assert_eq!(ma_slope.start_bars_ago, 5);
        assert_eq!(ma_slope.end_bars_ago, 0);
        assert_eq!(ma_slope.ma_type, MovingAverageType::Simple);
        assert!(!ma_slope.initialized);
        assert!(!ma_slope.has_inputs);
    }

    #[rstest]
    fn test_negative_index_conversion() {
        let ma_slope = MASlope::new(10, -5, Some(-10), Some(MovingAverageType::Simple), None);
        // start_bars_ago: 10 + (-5) = 5
        // end_bars_ago: 10 + (-10) = 0
        assert_eq!(ma_slope.start_bars_ago, 5);
        assert_eq!(ma_slope.end_bars_ago, 0);
    }

    #[rstest]
    fn test_one_value_input() {
        let mut ma_slope = MASlope::new(10, 5, Some(0), Some(MovingAverageType::Simple), None);
        ma_slope.update_raw(100.0);
        assert!(ma_slope.has_inputs());
        assert_eq!(ma_slope.value, 0.0); // Only 1 value, start_idx == end_idx after clamping
    }

    #[rstest]
    fn test_ma_slope_calculation() {
        let mut ma_slope = MASlope::new(5, 2, Some(0), Some(MovingAverageType::Simple), None);

        // Add values: 1, 2, 3, 4, 5
        for i in 1..=5 {
            ma_slope.update_raw(i as f64);
        }

        // After 5 inputs with SMA(5): MA values are [1, 1.5, 2, 2.5, 3]
        // ma_prices (newest first): [3, 2.5, 2, 1.5, 1]
        // start_idx = 2 (clamped to 2), end_idx = 0
        // slope = (ma_prices[0] - ma_prices[2]) / (2 - 0) = (3 - 2) / 2 = 0.5
        assert!(ma_slope.initialized());
        assert_eq!(ma_slope.value, 0.5);
    }

    #[rstest]
    fn test_handle_quote_tick(stub_quote: QuoteTick) {
        let mut ma_slope = MASlope::new(
            10,
            5,
            Some(0),
            Some(MovingAverageType::Simple),
            Some(PriceType::Mid),
        );
        ma_slope.handle_quote(&stub_quote);
        assert!(ma_slope.has_inputs());
    }

    #[rstest]
    fn test_handle_trade_tick(stub_trade: TradeTick) {
        let mut ma_slope = MASlope::new(10, 5, Some(0), Some(MovingAverageType::Simple), None);
        ma_slope.handle_trade(&stub_trade);
        assert!(ma_slope.has_inputs());
    }

    #[rstest]
    fn test_handle_bar(bar_ethusdt_binance_minute_bid: Bar) {
        let mut ma_slope = MASlope::new(10, 5, Some(0), Some(MovingAverageType::Simple), None);
        ma_slope.handle_bar(&bar_ethusdt_binance_minute_bid);
        assert!(ma_slope.has_inputs);
        assert!(!ma_slope.initialized);
    }

    #[rstest]
    fn test_reset() {
        let mut ma_slope = MASlope::new(10, 5, Some(0), Some(MovingAverageType::Simple), None);
        ma_slope.update_raw(100.0);
        ma_slope.update_raw(200.0);

        assert_eq!(ma_slope.count(), 2);
        assert_eq!(ma_slope.ma_prices.len(), 2);

        ma_slope.reset();

        assert_eq!(ma_slope.count(), 0);
        assert_eq!(ma_slope.value, 0.0);
        assert!(!ma_slope.has_inputs);
        assert!(!ma_slope.initialized);
        assert_eq!(ma_slope.ma_prices.len(), 0);
    }

    #[rstest]
    fn test_default_parameters() {
        let ma_slope = MASlope::new(10, 5, None, None, None);
        assert_eq!(ma_slope.end_bars_ago, 0);
        assert_eq!(ma_slope.ma_type, MovingAverageType::Simple);
        assert_eq!(ma_slope.price_type, PriceType::Last);
    }

    #[rstest]
    #[should_panic(expected = "`ma_period`")]
    fn new_panics_on_zero_period() {
        let _ = MASlope::new(0, 5, Some(0), None, None);
    }

    #[rstest]
    #[should_panic(expected = "start_bars_ago")]
    fn new_panics_when_start_not_greater_than_end() {
        let _ = MASlope::new(10, 2, Some(5), None, None);
    }

    #[rstest]
    fn test_clamp_index_behavior() {
        let ma_slope = MASlope::new(10, 5, Some(0), None, None);

        // Test clamping to valid range
        assert_eq!(ma_slope.clamp_index(0, 5), 0);
        assert_eq!(ma_slope.clamp_index(2, 5), 2);
        assert_eq!(ma_slope.clamp_index(4, 5), 4);
        assert_eq!(ma_slope.clamp_index(10, 5), 4); // Clamped to n-1

        // Edge case: empty range
        assert_eq!(ma_slope.clamp_index(5, 0), 0);
    }

    #[rstest]
    fn test_exponential_ma_type() {
        let mut ma_slope = MASlope::new(5, 2, Some(0), Some(MovingAverageType::Exponential), None);

        for i in 1..=10 {
            ma_slope.update_raw(i as f64);
        }

        assert!(ma_slope.initialized());
        // Slope should be positive since prices are increasing
        assert!(ma_slope.value > 0.0);
    }
}
