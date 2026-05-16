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

use std::fmt::Display;

use nautilus_model::{
    data::{Bar, QuoteTick, TradeTick},
    enums::PriceType,
};

use crate::indicator::{Indicator, MovingAverage};

/// The Zero-Lag Exponential Moving Average attempts to eliminate the lag
/// inherent in moving averages by incorporating a momentum term based on
/// the rate of change.
///
/// The ZLEMA uses a lag-adjusted value calculated from the current price
/// and a lagged price to reduce the impact of lag in traditional EMAs.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.indicators")
)]
pub struct ZeroLagExponentialMovingAverage {
    /// The rolling window period for the indicator (> 0).
    pub period: usize,
    /// The price type used for calculations.
    pub price_type: PriceType,
    /// The EMA smoothing constant.
    pub alpha: f64,
    /// The lag period.
    pub lag: usize,
    /// The last indicator value.
    pub value: f64,
    /// The input count for the indicator.
    pub count: usize,
    pub initialized: bool,
    has_inputs: bool,
    inputs: Vec<f64>,
}

impl Display for ZeroLagExponentialMovingAverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.period)
    }
}

impl Indicator for ZeroLagExponentialMovingAverage {
    fn name(&self) -> String {
        stringify!(ZeroLagExponentialMovingAverage).to_string()
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
        self.count = 0;
        self.has_inputs = false;
        self.initialized = false;
        self.inputs.clear();
    }
}

impl ZeroLagExponentialMovingAverage {
    /// Creates a new [`ZeroLagExponentialMovingAverage`] instance.
    ///
    /// # Panics
    ///
    /// Panics if `period` is not a positive integer (> 0).
    #[must_use]
    pub fn new(period: usize, price_type: Option<PriceType>) -> Self {
        assert!(
            period > 0,
            "ZeroLagExponentialMovingAverage::new → `period` must be positive (> 0); got {period}"
        );

        let lag = ((period - 1) as f64 / 2.0).ceil() as usize;

        Self {
            period,
            price_type: price_type.unwrap_or(PriceType::Last),
            alpha: 2.0 / (period as f64 + 1.0),
            lag,
            value: 0.0,
            count: 0,
            has_inputs: false,
            initialized: false,
            inputs: Vec::with_capacity(lag + 1),
        }
    }
}

impl MovingAverage for ZeroLagExponentialMovingAverage {
    fn value(&self) -> f64 {
        self.value
    }

    fn count(&self) -> usize {
        self.count
    }

    fn update_raw(&mut self, value: f64) {
        // Store the input value
        self.inputs.push(value);

        // Keep only lag+1 most recent values (circular buffer behavior)
        if self.inputs.len() > self.lag + 1 {
            self.inputs.remove(0);
        }

        // Check if this is the initial input or if we don't have enough history for lag
        if !self.has_inputs || self.inputs.len() <= self.lag {
            self.has_inputs = true;
            self.value = value;
        } else {
            // Get the oldest value (lagged value)
            let lagged_value = self.inputs[0];
            let adjusted_value = 2.0f64.mul_add(value, -lagged_value);

            // Calculate the zero-lag EMA
            self.value = self
                .alpha
                .mul_add(adjusted_value, (1.0 - self.alpha) * self.value);
        }

        self.count += 1;

        // Initialization logic
        if !self.initialized && self.count >= self.period {
            self.initialized = true;
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
        average::zlema::ZeroLagExponentialMovingAverage,
        indicator::{Indicator, MovingAverage},
        stubs::*,
    };

    #[rstest]
    fn test_zlema_initialized() {
        let zlema = ZeroLagExponentialMovingAverage::new(10, Some(PriceType::Mid));
        let display_str = format!("{zlema}");
        assert_eq!(display_str, "ZeroLagExponentialMovingAverage(10)");
        assert_eq!(zlema.period, 10);
        assert_eq!(zlema.price_type, PriceType::Mid);
        assert_eq!(zlema.alpha, 2.0 / 11.0);
        assert_eq!(zlema.lag, 5); // ceil((10-1)/2) = ceil(4.5) = 5
        assert!(!zlema.initialized);
        assert!(!zlema.has_inputs);
    }

    #[rstest]
    fn test_one_value_input() {
        let mut zlema = ZeroLagExponentialMovingAverage::new(10, None);
        zlema.update_raw(1.0);
        assert_eq!(zlema.count, 1);
        assert_eq!(zlema.value, 1.0);
        assert!(zlema.has_inputs());
    }

    #[rstest]
    fn test_zlema_update_raw() {
        let mut zlema = ZeroLagExponentialMovingAverage::new(10, None);

        // Feed sequential values and verify ZLEMA calculation
        for i in 1..=10 {
            zlema.update_raw(i as f64);
        }

        // After 10 inputs, should be initialized
        assert!(zlema.has_inputs());
        assert!(zlema.initialized());
        assert_eq!(zlema.count, 10);

        // Value should be positive and finite
        assert!(zlema.value > 0.0);
        assert!(zlema.value.is_finite());
    }

    #[rstest]
    fn test_reset() {
        let mut zlema = ZeroLagExponentialMovingAverage::new(10, None);
        zlema.update_raw(1.0);
        zlema.update_raw(2.0);

        assert_eq!(zlema.count, 2);
        assert_eq!(zlema.inputs.len(), 2);

        zlema.reset();

        assert_eq!(zlema.count, 0);
        assert_eq!(zlema.value, 0.0);
        assert!(!zlema.has_inputs);
        assert!(!zlema.initialized);
        assert_eq!(zlema.inputs.len(), 0);
    }

    #[rstest]
    fn test_handle_quote_tick(stub_quote: QuoteTick) {
        let mut zlema = ZeroLagExponentialMovingAverage::new(10, Some(PriceType::Mid));
        zlema.handle_quote(&stub_quote);
        assert!(zlema.has_inputs());
        assert_eq!(zlema.value, 1501.0);
    }

    #[rstest]
    fn test_handle_trade_tick(stub_trade: TradeTick) {
        let mut zlema = ZeroLagExponentialMovingAverage::new(10, None);
        zlema.handle_trade(&stub_trade);
        assert!(zlema.has_inputs());
        assert_eq!(zlema.value, 1500.0);
    }

    #[rstest]
    fn test_handle_bar(bar_ethusdt_binance_minute_bid: Bar) {
        let mut zlema = ZeroLagExponentialMovingAverage::new(10, None);
        zlema.handle_bar(&bar_ethusdt_binance_minute_bid);
        assert!(zlema.has_inputs);
        assert!(!zlema.initialized);
        assert_eq!(zlema.value, 1522.0);
    }

    #[rstest]
    fn test_period_one_behaviour() {
        let mut zlema = ZeroLagExponentialMovingAverage::new(1, None);
        assert_eq!(zlema.alpha, 1.0, "α must be 1 when period = 1");
        assert_eq!(zlema.lag, 0, "lag must be 0 when period = 1");

        zlema.update_raw(10.0);
        assert!(zlema.initialized());
        assert_eq!(zlema.value(), 10.0);

        zlema.update_raw(42.0);
        // With lag=0, inputs.len()=1 which is > lag, so we use ZLEMA formula
        // lagged_value = inputs[0] = 10.0
        // adjusted = 2*42 - 10 = 74
        // value = 1.0 * 74 + 0 * 10 = 74
        // But wait, after first update, inputs is cleared and only keeps lag+1=1 value
        // Let me trace through: after first update_raw(10), inputs=[10], count=1, value=10
        // Then update_raw(42): inputs.push(42) -> [10,42], len=2 > lag+1=1, remove(0) -> [42]
        // lagged_value = inputs[0] = 42, adjusted = 2*42 - 42 = 42
        // Actually no - after push and before remove: inputs=[10,42], len=2, lag+1=1
        // So we remove(0) first: inputs=[42]
        // Wait, the order matters. Let me check the code flow again.

        // In update_raw: push(42) -> [10, 42], then if len > lag+1: remove(0) -> [42]
        // Then check: len=1 <= lag=0? No, 1 > 0
        // So lagged_value = inputs[0] = 42
        // adjusted = 2*42 - 42 = 42
        // value = 1.0 * 42 + 0 * 10 = 42
        assert_eq!(zlema.value(), 42.0);
    }

    #[rstest]
    fn test_default_price_type_is_last() {
        let zlema = ZeroLagExponentialMovingAverage::new(3, None);
        assert_eq!(
            zlema.price_type,
            PriceType::Last,
            "`price_type` default mismatch"
        );
    }

    #[rstest]
    fn test_lag_calculation() {
        let zlema5 = ZeroLagExponentialMovingAverage::new(5, None);
        assert_eq!(zlema5.lag, 2); // ceil((5-1)/2) = ceil(2) = 2

        let zlema10 = ZeroLagExponentialMovingAverage::new(10, None);
        assert_eq!(zlema10.lag, 5); // ceil((10-1)/2) = ceil(4.5) = 5

        let zlema20 = ZeroLagExponentialMovingAverage::new(20, None);
        assert_eq!(zlema20.lag, 10); // ceil((20-1)/2) = ceil(9.5) = 10
    }

    #[rstest]
    fn test_nan_poisoning_and_reset_recovery() {
        let mut zlema = ZeroLagExponentialMovingAverage::new(4, None);
        for x in 0..3 {
            zlema.update_raw(f64::from(x));
            assert!(zlema.value().is_finite());
        }

        zlema.update_raw(f64::NAN);
        assert!(zlema.value().is_nan());

        zlema.update_raw(123.456);
        assert!(zlema.value().is_nan());

        zlema.reset();
        assert!(!zlema.has_inputs());
        zlema.update_raw(7.0);
        assert_eq!(zlema.value(), 7.0);
        assert!(zlema.value().is_finite());
    }

    #[rstest]
    fn test_reset_without_inputs_is_safe() {
        let mut zlema = ZeroLagExponentialMovingAverage::new(8, None);
        zlema.reset();
        assert!(!zlema.has_inputs());
        assert_eq!(zlema.count(), 0);
        assert!(!zlema.initialized());
    }

    #[rstest]
    #[should_panic(expected = "`period`")]
    fn new_panics_on_zero_period() {
        let _ = ZeroLagExponentialMovingAverage::new(0, None);
    }
}
