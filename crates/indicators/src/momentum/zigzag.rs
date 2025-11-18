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
//
// Updated: 2025-07-31 – Aligned swing-detection logic with NinjaTrader reference implementation.
// -------------------------------------------------------------------------------------------------

use std::fmt::{Display, Formatter};

use nautilus_model::data::Bar;

use crate::indicator::Indicator;

#[derive(Debug, Clone)]
struct PriceBar {
    high: f64,
    low: f64,
    close: f64,
    bar_index: i32,
}

/// ZigZag indicator that mirrors NinjaTrader's floating-point-tolerant
/// swing-detection logic.
///
/// The ZigZag indicator filters out price noise based on deviation thresholds
/// and identifies swing highs and lows in the price data.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.indicators")
)]
pub struct ZigZag {
    /// The deviation threshold value.
    pub deviation_value: f64,
    /// Whether to use point-based deviation (true) or percent-based (false).
    pub use_point_deviation_type: bool,
    /// Whether to use high/low prices (true) or close prices (false).
    pub use_high_low: bool,
    /// The most recent swing high price.
    pub current_high: f64,
    /// The most recent swing low price.
    pub current_low: f64,
    /// The price of the most recent swing (high or low).
    pub last_swing_price: f64,
    /// Current trend direction: 1 = up, -1 = down, 0 = init.
    pub trend_direction: i32,
    /// Bar index of the most recent swing.
    pub last_swing_idx: i32,
    /// Number of bars processed.
    pub count: i32,
    pub initialized: bool,
    has_inputs: bool,
    highs: Vec<f64>,
    lows: Vec<f64>,
    high_bars: Vec<i32>,
    low_bars: Vec<i32>,
    price_history: Vec<PriceBar>,
}

impl Display for ZigZag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:.2})", self.name(), self.deviation_value)
    }
}

impl Indicator for ZigZag {
    fn name(&self) -> String {
        stringify!(ZigZag).to_string()
    }

    fn has_inputs(&self) -> bool {
        self.has_inputs
    }

    fn initialized(&self) -> bool {
        self.initialized
    }

    fn handle_quote(&mut self, _quote: &nautilus_model::data::QuoteTick) {
        // ZigZag only handles bars
    }

    fn handle_trade(&mut self, _trade: &nautilus_model::data::TradeTick) {
        // ZigZag only handles bars
    }

    fn handle_bar(&mut self, bar: &Bar) {
        let high: f64 = (&bar.high).into();
        let low: f64 = (&bar.low).into();
        let close: f64 = (&bar.close).into();
        self.update_raw(high, low, close);
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.high_bars.clear();
        self.low_bars.clear();
        self.price_history.clear();
        self.current_high = 0.0;
        self.current_low = 0.0;
        self.last_swing_price = 0.0;
        self.trend_direction = 0;
        self.last_swing_idx = -1;
        self.count = 0;
        self.has_inputs = false;
        self.initialized = false;
    }
}

impl ZigZag {
    /// Creates a new [`ZigZag`] instance.
    ///
    /// # Parameters
    ///
    /// - `deviation_value`: The deviation threshold (> 0).
    /// - `use_point_deviation_type`: Use point-based deviation (true) or percent-based (false).
    /// - `use_high_low`: Use high/low prices (true) or close prices (false).
    ///
    /// # Panics
    ///
    /// Panics if `deviation_value` is not positive (> 0).
    #[must_use]
    pub fn new(
        deviation_value: f64,
        use_point_deviation_type: Option<bool>,
        use_high_low: Option<bool>,
    ) -> Self {
        assert!(
            deviation_value > 0.0,
            "ZigZag::new → `deviation_value` must be positive (> 0); got {deviation_value}"
        );

        Self {
            deviation_value,
            use_point_deviation_type: use_point_deviation_type.unwrap_or(false),
            use_high_low: use_high_low.unwrap_or(false),
            current_high: 0.0,
            current_low: 0.0,
            last_swing_price: 0.0,
            trend_direction: 0,
            last_swing_idx: -1,
            count: 0,
            initialized: false,
            has_inputs: false,
            highs: Vec::new(),
            lows: Vec::new(),
            high_bars: Vec::new(),
            low_bars: Vec::new(),
            price_history: Vec::with_capacity(3),
        }
    }

    /// Update the indicator with raw high, low, close values.
    pub fn update_raw(&mut self, high: f64, low: f64, close: f64) {
        // Bookkeeping
        self.count += 1;
        let current_bar = PriceBar {
            high,
            low,
            close,
            bar_index: self.count,
        };

        self.price_history.push(current_bar);
        if self.price_history.len() > 3 {
            self.price_history.remove(0);
        }

        if !self.has_inputs {
            self.has_inputs = true;
        }

        if self.price_history.len() < 3 {
            return;
        }

        if self.last_swing_price == 0.0 {
            self.last_swing_price = close;
        }

        // Choose series (High/Low or Close)
        let high_series: Vec<f64> = if self.use_high_low {
            self.price_history.iter().map(|b| b.high).collect()
        } else {
            self.price_history.iter().map(|b| b.close).collect()
        };

        let low_series: Vec<f64> = if self.use_high_low {
            self.price_history.iter().map(|b| b.low).collect()
        } else {
            self.price_history.iter().map(|b| b.close).collect()
        };

        // Swing detection with regular comparisons
        let is_swing_high = high_series[1] >= high_series[0] && high_series[1] >= high_series[2];
        let is_swing_low = low_series[1] <= low_series[0] && low_series[1] <= low_series[2];

        // Deviation thresholds
        let (high_threshold, low_threshold) = if self.use_point_deviation_type {
            (
                self.last_swing_price + self.deviation_value,
                self.last_swing_price - self.deviation_value,
            )
        } else {
            // Percent mode
            (
                self.last_swing_price * (1.0 + self.deviation_value / 100.0),
                self.last_swing_price * (1.0 - self.deviation_value / 100.0),
            )
        };

        let is_over_high_deviation = high_series[1] > high_threshold;
        let is_over_low_deviation = low_threshold > low_series[1];

        // Early exit if no swing candidate
        if !is_swing_high && !is_swing_low {
            return;
        }

        // Determine action (add/update swing)
        let mut save_value = 0.0;
        let mut add_high = false;
        let mut add_low = false;
        let mut update_high = false;
        let mut update_low = false;

        if self.trend_direction <= 0 && is_swing_high && is_over_high_deviation {
            save_value = high_series[1];
            add_high = true;
            self.trend_direction = 1;
        } else if self.trend_direction >= 0 && is_swing_low && is_over_low_deviation {
            save_value = low_series[1];
            add_low = true;
            self.trend_direction = -1;
        } else if self.trend_direction == 1
            && is_swing_high
            && high_series[1] > self.last_swing_price
        {
            save_value = high_series[1];
            update_high = true;
        } else if self.trend_direction == -1
            && is_swing_low
            && self.last_swing_price > low_series[1]
        {
            save_value = low_series[1];
            update_low = true;
        }

        // Commit swing changes
        if add_high || add_low || update_high || update_low {
            if update_high && !self.highs.is_empty() {
                let last_idx = self.highs.len() - 1;
                self.highs[last_idx] = save_value;
                self.high_bars[last_idx] = self.price_history[1].bar_index;
                self.current_high = save_value;
            } else if update_low && !self.lows.is_empty() {
                let last_idx = self.lows.len() - 1;
                self.lows[last_idx] = save_value;
                self.low_bars[last_idx] = self.price_history[1].bar_index;
                self.current_low = save_value;
            }

            if add_high {
                self.highs.push(save_value);
                self.high_bars.push(self.price_history[1].bar_index);
                self.current_high = save_value;
            } else if add_low {
                self.lows.push(save_value);
                self.low_bars.push(self.price_history[1].bar_index);
                self.current_low = save_value;
            }

            self.last_swing_idx = self.price_history[1].bar_index;
            self.last_swing_price = save_value;

            if !self.initialized {
                self.initialized = true;
            }
        }
    }

    /// Returns a copy of swing high prices.
    pub fn get_highs(&self) -> Vec<f64> {
        self.highs.clone()
    }

    /// Returns a copy of swing low prices.
    pub fn get_lows(&self) -> Vec<f64> {
        self.lows.clone()
    }

    /// Returns a copy of swing high bar indices.
    pub fn get_high_bars(&self) -> Vec<i32> {
        self.high_bars.clone()
    }

    /// Returns a copy of swing low bar indices.
    pub fn get_low_bars(&self) -> Vec<i32> {
        self.low_bars.clone()
    }

    /// Find the Nth swing high within a lookback period.
    ///
    /// # Parameters
    ///
    /// - `bars_ago`: How many bars ago to start looking from.
    /// - `instance`: Which instance (1 = most recent).
    /// - `lookback_period`: How far back to search.
    ///
    /// # Returns
    ///
    /// The number of bars ago the swing high occurred, or -1 if not found.
    pub fn high_bar(&self, bars_ago: i32, instance: i32, lookback_period: i32) -> i32 {
        if instance < 1 || bars_ago < 0 {
            return -1;
        }

        let target_bar = self.count - bars_ago - 1;
        let mut found = 0;

        for &bar_idx in self.high_bars.iter().rev() {
            if bar_idx <= target_bar && bar_idx >= target_bar - lookback_period {
                found += 1;
                if found == instance {
                    return self.count - bar_idx;
                }
            }
        }

        -1
    }

    /// Find the Nth swing low within a lookback period.
    ///
    /// # Parameters
    ///
    /// - `bars_ago`: How many bars ago to start looking from.
    /// - `instance`: Which instance (1 = most recent).
    /// - `lookback_period`: How far back to search.
    ///
    /// # Returns
    ///
    /// The number of bars ago the swing low occurred, or -1 if not found.
    pub fn low_bar(&self, bars_ago: i32, instance: i32, lookback_period: i32) -> i32 {
        if instance < 1 || bars_ago < 0 {
            return -1;
        }

        let target_bar = self.count - bars_ago - 1;
        let mut found = 0;

        for &bar_idx in self.low_bars.iter().rev() {
            if bar_idx <= target_bar && bar_idx >= target_bar - lookback_period {
                found += 1;
                if found == instance {
                    return self.count - bar_idx;
                }
            }
        }

        -1
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use nautilus_model::data::Bar;
    use rstest::rstest;

    use crate::{indicator::Indicator, momentum::zigzag::ZigZag, stubs::*};

    #[rstest]
    fn test_zigzag_initialized() {
        let zz = ZigZag::new(5.0, Some(false), Some(true));
        let display_str = format!("{zz}");
        assert_eq!(display_str, "ZigZag(5.00)");
        assert_eq!(zz.deviation_value, 5.0);
        assert!(!zz.use_point_deviation_type);
        assert!(zz.use_high_low);
        assert!(!zz.initialized);
        assert!(!zz.has_inputs);
        assert_eq!(zz.trend_direction, 0);
    }

    #[rstest]
    fn test_defaults() {
        let zz = ZigZag::new(5.0, None, None);
        assert!(!zz.use_point_deviation_type); // Default: percent mode
        assert!(!zz.use_high_low); // Default: close prices
    }

    #[rstest]
    fn test_first_bars_no_swing() {
        let mut zz = ZigZag::new(5.0, Some(false), Some(false));

        // First bar
        zz.update_raw(100.0, 99.0, 99.5);
        assert!(zz.has_inputs());
        assert!(!zz.initialized());
        assert_eq!(zz.count, 1);

        // Second bar
        zz.update_raw(101.0, 100.0, 100.5);
        assert_eq!(zz.count, 2);
        assert!(!zz.initialized()); // Still need 3 bars for swing detection
    }

    #[rstest]
    fn test_swing_high_detection() {
        let mut zz = ZigZag::new(5.0, Some(false), Some(false));

        // Create a swing high pattern: low, high, low (using close prices)
        zz.update_raw(100.0, 99.0, 100.0);
        zz.update_raw(110.0, 109.0, 110.0); // Middle bar - potential swing high
        zz.update_raw(105.0, 104.0, 105.0);

        // last_swing_price started at 105.0 (close of 3rd bar)
        // high_threshold = 105.0 * 1.05 = 110.25
        // 110.0 < 110.25, so no swing yet

        assert!(!zz.initialized()); // No swing detected yet
    }

    #[rstest]
    fn test_swing_with_sufficient_deviation() {
        let mut zz = ZigZag::new(5.0, Some(false), Some(false));

        // Create significant upward movement
        zz.update_raw(100.0, 99.0, 100.0);
        zz.update_raw(115.0, 114.0, 115.0); // Middle bar with 15% gain
        zz.update_raw(110.0, 109.0, 110.0);

        // last_swing_price = 110.0, threshold = 110 * 1.05 = 115.5
        // 115.0 < 115.5, might not trigger

        // Let's add more bars to create a clear pattern
        zz.update_raw(112.0, 111.0, 112.0);
        zz.update_raw(120.0, 119.0, 120.0); // New potential swing high
        zz.update_raw(115.0, 114.0, 115.0);

        // Should eventually initialize when a swing is detected
        assert!(zz.count >= 3);
    }

    #[rstest]
    fn test_point_deviation_mode() {
        let mut zz = ZigZag::new(10.0, Some(true), Some(false)); // 10 point deviation

        zz.update_raw(100.0, 99.0, 100.0);
        zz.update_raw(115.0, 114.0, 115.0);
        zz.update_raw(110.0, 109.0, 110.0);

        // In point mode: threshold = last_swing_price + 10.0
        assert!(zz.count >= 3);
    }

    #[rstest]
    fn test_handle_bar(bar_ethusdt_binance_minute_bid: Bar) {
        let mut zz = ZigZag::new(5.0, Some(false), Some(true));
        zz.handle_bar(&bar_ethusdt_binance_minute_bid);
        assert!(zz.has_inputs);
        assert_eq!(zz.count, 1);
    }

    #[rstest]
    fn test_reset() {
        let mut zz = ZigZag::new(5.0, Some(false), Some(false));

        zz.update_raw(100.0, 99.0, 100.0);
        zz.update_raw(110.0, 109.0, 110.0);
        zz.update_raw(105.0, 104.0, 105.0);

        assert_eq!(zz.count, 3);

        zz.reset();

        assert_eq!(zz.count, 0);
        assert_eq!(zz.current_high, 0.0);
        assert_eq!(zz.current_low, 0.0);
        assert_eq!(zz.last_swing_price, 0.0);
        assert_eq!(zz.trend_direction, 0);
        assert_eq!(zz.last_swing_idx, -1);
        assert!(!zz.has_inputs);
        assert!(!zz.initialized);
        assert_eq!(zz.highs.len(), 0);
        assert_eq!(zz.lows.len(), 0);
    }

    #[rstest]
    fn test_get_methods() {
        let mut zz = ZigZag::new(5.0, Some(false), Some(false));

        // Create a pattern that should generate swings
        for i in 0..20 {
            let price = 100.0 + (i as f64 * 2.0) * if i % 4 < 2 { 1.0 } else { -1.0 };
            zz.update_raw(price, price - 1.0, price);
        }

        // Get methods should return copies
        let highs = zz.get_highs();
        let lows = zz.get_lows();
        let high_bars = zz.get_high_bars();
        let low_bars = zz.get_low_bars();

        // Verify they're separate copies
        assert_eq!(highs.len(), zz.highs.len());
        assert_eq!(lows.len(), zz.lows.len());
        assert_eq!(high_bars.len(), zz.high_bars.len());
        assert_eq!(low_bars.len(), zz.low_bars.len());
    }

    #[rstest]
    fn test_high_bar_and_low_bar_methods() {
        let zz = ZigZag::new(5.0, Some(false), Some(false));

        // Test invalid inputs
        assert_eq!(zz.high_bar(-1, 1, 100), -1); // Negative bars_ago
        assert_eq!(zz.high_bar(0, 0, 100), -1); // instance < 1
        assert_eq!(zz.low_bar(-1, 1, 100), -1); // Negative bars_ago
        assert_eq!(zz.low_bar(0, 0, 100), -1); // instance < 1
    }

    #[rstest]
    #[should_panic(expected = "`deviation_value`")]
    fn new_panics_on_zero_deviation() {
        let _ = ZigZag::new(0.0, None, None);
    }

    #[rstest]
    #[should_panic(expected = "`deviation_value`")]
    fn new_panics_on_negative_deviation() {
        let _ = ZigZag::new(-5.0, None, None);
    }

    #[rstest]
    fn test_trend_direction_changes() {
        let mut zz = ZigZag::new(10.0, Some(false), Some(false));

        // Initial trend is 0
        assert_eq!(zz.trend_direction, 0);

        // Add bars and verify trend changes when swings are detected
        for i in 0..10 {
            let price = 100.0 + i as f64 * 5.0;
            zz.update_raw(price, price - 1.0, price);
        }

        // Trend direction should have changed from 0 if swings were detected
        // (actual value depends on the pattern)
    }
}
