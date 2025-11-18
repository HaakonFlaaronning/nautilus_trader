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

use std::fmt::{Display, Formatter};

use nautilus_model::{
    data::{Bar, QuoteTick, TradeTick},
    enums::PriceType,
};

use crate::{
    average::{MovingAverageFactory, MovingAverageType},
    indicator::{Indicator, MovingAverage},
};

/// WaveTrend is a fast moving MACD-based oscillator that is quick to react
/// to crossovers.
///
/// It calculates the difference between price and exponential moving averages
/// to create an oscillator with two lines: a fast line (TCI) and a slow line.
///
/// Based on the WaveTrend oscillator by LazyBear (TradingView).
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.indicators", unsendable)
)]
pub struct WaveTrendOscillator {
    /// The channel length period for EMA calculations (> 0).
    pub channel_length: usize,
    /// The average length period for TCI calculation (> 0).
    pub average_length: usize,
    /// The price type used for calculations.
    pub price_type: PriceType,
    /// The fast WaveTrend line (TCI).
    pub wt_fast: f64,
    /// The slow WaveTrend line.
    pub wt_slow: f64,
    pub initialized: bool,
    has_inputs: bool,
    esa: Box<dyn MovingAverage + Send + 'static>,
    d_ema: Box<dyn MovingAverage + Send + 'static>,
    tci_ema: Box<dyn MovingAverage + Send + 'static>,
    wt_slow_ma: Box<dyn MovingAverage + Send + 'static>,
}

impl Display for WaveTrendOscillator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({},{})",
            self.name(),
            self.channel_length,
            self.average_length
        )
    }
}

impl Indicator for WaveTrendOscillator {
    fn name(&self) -> String {
        stringify!(WaveTrendOscillator).to_string()
    }

    fn has_inputs(&self) -> bool {
        self.has_inputs
    }

    fn initialized(&self) -> bool {
        self.initialized
    }

    fn handle_quote(&mut self, quote: &QuoteTick) {
        let price: f64 = quote.extract_price(self.price_type).into();
        self.update_raw(price, price, price);
    }

    fn handle_trade(&mut self, trade: &TradeTick) {
        let price: f64 = (&trade.price).into();
        self.update_raw(price, price, price);
    }

    fn handle_bar(&mut self, bar: &Bar) {
        let high: f64 = (&bar.high).into();
        let low: f64 = (&bar.low).into();
        let close: f64 = (&bar.close).into();
        self.update_raw(high, low, close);
    }

    fn reset(&mut self) {
        self.wt_fast = 0.0;
        self.wt_slow = 0.0;
        self.esa.reset();
        self.d_ema.reset();
        self.tci_ema.reset();
        self.wt_slow_ma.reset();
        self.has_inputs = false;
        self.initialized = false;
    }
}

impl WaveTrendOscillator {
    /// Creates a new [`WaveTrendOscillator`] instance.
    ///
    /// # Parameters
    ///
    /// - `channel_length`: The channel length period for EMA calculations (> 0).
    /// - `average_length`: The average length period for TCI calculation (> 0).
    /// - `price_type`: The price type for extracting values (default: LAST).
    ///
    /// # Panics
    ///
    /// Panics if `channel_length` is not positive (> 0).
    /// Panics if `average_length` is not positive (> 0).
    #[must_use]
    pub fn new(
        channel_length: usize,
        average_length: usize,
        price_type: Option<PriceType>,
    ) -> Self {
        assert!(
            channel_length > 0,
            "WaveTrendOscillator::new → `channel_length` must be positive (> 0); got {channel_length}"
        );
        assert!(
            average_length > 0,
            "WaveTrendOscillator::new → `average_length` must be positive (> 0); got {average_length}"
        );

        Self {
            channel_length,
            average_length,
            price_type: price_type.unwrap_or(PriceType::Last),
            wt_fast: 0.0,
            wt_slow: 0.0,
            initialized: false,
            has_inputs: false,
            esa: MovingAverageFactory::create(MovingAverageType::Exponential, channel_length),
            d_ema: MovingAverageFactory::create(MovingAverageType::Exponential, channel_length),
            tci_ema: MovingAverageFactory::create(MovingAverageType::Exponential, average_length),
            wt_slow_ma: MovingAverageFactory::create(MovingAverageType::Simple, 4),
        }
    }

    /// Update the indicator with raw high, low, close values.
    pub fn update_raw(&mut self, high: f64, low: f64, close: f64) {
        // Check if first input
        if !self.has_inputs {
            self.has_inputs = true;
        }

        // Calculate typical price (average price)
        let ap = (high + low + close) / 3.0;

        // Update ESA (EMA of typical price)
        self.esa.update_raw(ap);

        // Calculate absolute difference and update d_ema
        let d_absolute = (ap - self.esa.value()).abs();
        self.d_ema.update_raw(d_absolute);

        // Calculate Channel Index (CI)
        let ci = if self.d_ema.value() != 0.0 {
            (ap - self.esa.value()) / (0.015 * self.d_ema.value())
        } else {
            0.0
        };

        // Update TCI (EMA of CI) - this is the fast WaveTrend line
        self.tci_ema.update_raw(ci);
        self.wt_fast = self.tci_ema.value();

        // Update slow WaveTrend line (SMA of fast line)
        self.wt_slow_ma.update_raw(self.wt_fast);
        self.wt_slow = self.wt_slow_ma.value();

        // Initialization logic
        if !self.initialized
            && self.esa.initialized()
            && self.d_ema.initialized()
            && self.tci_ema.initialized()
            && self.wt_slow_ma.initialized()
        {
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
        indicator::Indicator, momentum::wavetrend::WaveTrendOscillator, stubs::*,
        testing::approx_equal,
    };

    #[rstest]
    fn test_wavetrend_initialized() {
        let wt = WaveTrendOscillator::new(10, 21, Some(PriceType::Mid));
        let display_str = format!("{wt}");
        assert_eq!(display_str, "WaveTrendOscillator(10,21)");
        assert_eq!(wt.channel_length, 10);
        assert_eq!(wt.average_length, 21);
        assert_eq!(wt.price_type, PriceType::Mid);
        assert!(!wt.initialized);
        assert!(!wt.has_inputs);
        assert_eq!(wt.wt_fast, 0.0);
        assert_eq!(wt.wt_slow, 0.0);
    }

    #[rstest]
    fn test_one_value_input() {
        let mut wt = WaveTrendOscillator::new(10, 21, None);
        wt.update_raw(100.0, 99.0, 99.5);
        assert!(wt.has_inputs());
        assert!(!wt.initialized());
    }

    #[rstest]
    fn test_wavetrend_calculation() {
        let mut wt = WaveTrendOscillator::new(10, 21, None);

        // Add some price data with upward trend
        for i in 1..=30 {
            let price = 100.0 + i as f64;
            wt.update_raw(price + 1.0, price - 1.0, price);
        }

        // After 30 inputs, should be initialized (needs max(10, 21, 4) = 21 inputs)
        assert!(wt.initialized());

        // With upward trending prices, wt_fast should be positive
        assert!(wt.wt_fast > 0.0);

        // wt_slow should lag behind wt_fast
        assert!(wt.wt_slow.abs() > 0.0);
    }

    #[rstest]
    fn test_initialization_threshold() {
        let mut wt = WaveTrendOscillator::new(10, 21, None);

        // Need max(channel_length=10, average_length=21, 4) = 21 inputs to initialize
        for _ in 1..21 {
            wt.update_raw(100.0, 99.0, 99.5);
            assert!(!wt.initialized());
        }

        // 21st input should initialize
        wt.update_raw(100.0, 99.0, 99.5);
        assert!(wt.initialized());
    }

    #[rstest]
    fn test_handle_quote_tick(stub_quote: QuoteTick) {
        let mut wt = WaveTrendOscillator::new(10, 21, Some(PriceType::Mid));
        wt.handle_quote(&stub_quote);
        assert!(wt.has_inputs());
    }

    #[rstest]
    fn test_handle_trade_tick(stub_trade: TradeTick) {
        let mut wt = WaveTrendOscillator::new(10, 21, None);
        wt.handle_trade(&stub_trade);
        assert!(wt.has_inputs());
    }

    #[rstest]
    fn test_handle_bar(bar_ethusdt_binance_minute_bid: Bar) {
        let mut wt = WaveTrendOscillator::new(10, 21, None);
        wt.handle_bar(&bar_ethusdt_binance_minute_bid);
        assert!(wt.has_inputs);
        assert!(!wt.initialized);
    }

    #[rstest]
    fn test_reset() {
        let mut wt = WaveTrendOscillator::new(10, 21, None);

        for i in 1..=25 {
            let price = 100.0 + i as f64;
            wt.update_raw(price, price, price);
        }

        assert!(wt.initialized());
        assert!(wt.wt_fast != 0.0);

        wt.reset();

        assert_eq!(wt.wt_fast, 0.0);
        assert_eq!(wt.wt_slow, 0.0);
        assert!(!wt.has_inputs);
        assert!(!wt.initialized);
    }

    #[rstest]
    fn test_default_price_type() {
        let wt = WaveTrendOscillator::new(10, 21, None);
        assert_eq!(wt.price_type, PriceType::Last);
    }

    #[rstest]
    #[should_panic(expected = "`channel_length`")]
    fn new_panics_on_zero_channel_length() {
        let _ = WaveTrendOscillator::new(0, 21, None);
    }

    #[rstest]
    #[should_panic(expected = "`average_length`")]
    fn new_panics_on_zero_average_length() {
        let _ = WaveTrendOscillator::new(10, 0, None);
    }

    #[rstest]
    fn test_division_by_zero_protection() {
        let mut wt = WaveTrendOscillator::new(10, 21, None);

        // Feed constant prices which should result in d_ema.value() approaching 0
        for _ in 1..=30 {
            wt.update_raw(100.0, 100.0, 100.0);
        }

        // Should not panic and wt_fast should be finite
        assert!(wt.wt_fast.is_finite());
        assert!(approx_equal(wt.wt_fast, 0.0));
    }

    #[rstest]
    fn test_oscillating_prices() {
        let mut wt = WaveTrendOscillator::new(10, 21, None);

        // Feed oscillating prices
        for i in 1..=50 {
            let price = 100.0 + ((i as f64 * 0.5).sin() * 10.0);
            wt.update_raw(price + 1.0, price - 1.0, price);
        }

        assert!(wt.initialized());
        // Values should oscillate around zero
        assert!(wt.wt_fast.abs() < 100.0); // Sanity check
    }
}
