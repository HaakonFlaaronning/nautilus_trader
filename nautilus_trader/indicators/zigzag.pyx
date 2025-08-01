# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software distributed under
#  the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
#  either express or implied. See the License for the specific language governing permissions and
#  limitations under the License.
#
#  Updated: 2025‑07‑31 – Aligned swing‑detection logic with NinjaTrader reference implementation.
# -------------------------------------------------------------------------------------------------

from libc.math cimport fabs

from nautilus_trader.core.correctness cimport Condition
from nautilus_trader.indicators.base.indicator cimport Indicator
from nautilus_trader.model.data cimport Bar


cdef class ZigZag(Indicator):
    """
    ZigZag indicator that now mirrors NinjaTrader's floating‑point‑tolerant swing‑detection logic.
    """

    def __init__(
        self,
        double deviation_value,
        bint use_point_deviation_type=False,
        bint use_high_low=False,
    ):
        Condition.positive(deviation_value, "deviation_value")

        params = [
            deviation_value,
            use_point_deviation_type,
            use_high_low,
        ]
        super().__init__(params=params)

        self.deviation_value = deviation_value
        self.use_point_deviation_type = use_point_deviation_type
        self.use_high_low = use_high_low

        # Internal state ------------------------------------------------
        self._highs = []
        self._lows = []
        self._high_bars = []
        self._low_bars = []
        self._price_history = []   # last three bars

        self.current_high = 0.0
        self.current_low = 0.0
        self.last_swing_price = 0.0
        self.trend_direction = 0      # 1 = up, -1 = down, 0 = init
        self.last_swing_idx = -1
        self.count = 0                # processed bars

    # ------------------------------------------------------------------
    #                         public interface
    # ------------------------------------------------------------------
    cpdef void handle_bar(self, Bar bar):
        Condition.not_none(bar, "bar")

        self.update_raw(
            bar.high.as_double(),
            bar.low.as_double(),
            bar.close.as_double(),
        )

    cpdef void update_raw(self, double high, double low, double close):
        # --------------------------------------------------------------
        # bookkeeping
        # --------------------------------------------------------------
        self.count += 1
        cdef dict current_bar = {
            'high': high,
            'low': low,
            'close': close,
            'bar_index': self.count,
        }
        self._price_history.append(current_bar)
        if len(self._price_history) > 3:
            self._price_history.pop(0)

        if not self.has_inputs:
            self._set_has_inputs(True)
        if len(self._price_history) < 3:
            return

        if self.last_swing_price == 0.0:
            self.last_swing_price = close

        # --------------------------------------------------------------
        # choose series (High/Low or Close)
        # --------------------------------------------------------------
        cdef list high_series = []
        cdef list low_series = []
        for i in range(len(self._price_history)):
            if self.use_high_low:
                high_series.append(self._price_history[i]['high'])
                low_series.append(self._price_history[i]['low'])
            else:
                high_series.append(self._price_history[i]['close'])
                low_series.append(self._price_history[i]['close'])

        # --------------------------------------------------------------
        # swing‑detection with regular comparisons
        # --------------------------------------------------------------
        cdef bint is_swing_high = (
            high_series[1] >= high_series[0] and
            high_series[1] >= high_series[2]
        )
        cdef bint is_swing_low = (
            low_series[1] <= low_series[0] and
            low_series[1] <= low_series[2]
        )

        # deviation thresholds
        cdef double high_threshold
        cdef double low_threshold
        if self.use_point_deviation_type:
            high_threshold = self.last_swing_price + self.deviation_value
            low_threshold = self.last_swing_price - self.deviation_value
        else:  # percent
            high_threshold = self.last_swing_price * (1.0 + self.deviation_value / 100.0)
            low_threshold = self.last_swing_price * (1.0 - self.deviation_value / 100.0)

        cdef bint is_over_high_deviation = high_series[1] > high_threshold
        cdef bint is_over_low_deviation = low_threshold > low_series[1]

        # --------------------------------------------------------------
        # early exit if no swing candidate
        # --------------------------------------------------------------
        if not is_swing_high and not is_swing_low:
            return

        # --------------------------------------------------------------
        # determine action (add/update swing)
        # --------------------------------------------------------------
        cdef double save_value = 0.0
        cdef bint add_high = False
        cdef bint add_low = False
        cdef bint update_high = False
        cdef bint update_low = False

        if self.trend_direction <= 0 and is_swing_high and is_over_high_deviation:
            save_value = high_series[1]
            add_high = True
            self.trend_direction = 1
        elif self.trend_direction >= 0 and is_swing_low and is_over_low_deviation:
            save_value = low_series[1]
            add_low = True
            self.trend_direction = -1
        elif self.trend_direction == 1 and is_swing_high and high_series[1] > self.last_swing_price:
            save_value = high_series[1]
            update_high = True
        elif self.trend_direction == -1 and is_swing_low and self.last_swing_price > low_series[1]:
            save_value = low_series[1]
            update_low = True

        # --------------------------------------------------------------
        # commit swing changes
        # --------------------------------------------------------------
        if add_high or add_low or update_high or update_low:
            if update_high and len(self._highs) > 0:
                self._highs[-1] = save_value
                self._high_bars[-1] = self._price_history[1]['bar_index']
                self.current_high = save_value   # refreshed
            elif update_low and len(self._lows) > 0:
                self._lows[-1] = save_value
                self._low_bars[-1] = self._price_history[1]['bar_index']
                self.current_low = save_value    # refreshed

            if add_high:
                self._highs.append(save_value)
                self._high_bars.append(self._price_history[1]['bar_index'])
                self.current_high = save_value
            elif add_low:
                self._lows.append(save_value)
                self._low_bars.append(self._price_history[1]['bar_index'])
                self.current_low = save_value

            self.last_swing_idx = self._price_history[1]['bar_index']
            self.last_swing_price = save_value

            if not self.initialized:
                self._set_initialized(True)

    # ------------------------------------------------------------------
    #                      public getters (unchanged)
    # ------------------------------------------------------------------
    cpdef list get_highs(self):
        return self._highs.copy()

    cpdef list get_lows(self):
        return self._lows.copy()

    cpdef list get_high_bars(self):
        return self._high_bars.copy()

    cpdef list get_low_bars(self):
        return self._low_bars.copy()

    cpdef int high_bar(self, int bars_ago, int instance, int lookback_period):
        if instance < 1 or bars_ago < 0:
            return -1
        cdef int target_bar = self.count - bars_ago - 1
        cdef int found = 0
        for i in range(len(self._high_bars) - 1, -1, -1):
            if self._high_bars[i] <= target_bar and self._high_bars[i] >= target_bar - lookback_period:
                found += 1
                if found == instance:
                    return self.count - self._high_bars[i]
        return -1

    cpdef int low_bar(self, int bars_ago, int instance, int lookback_period):
        if instance < 1 or bars_ago < 0:
            return -1
        cdef int target_bar = self.count - bars_ago - 1
        cdef int found = 0
        for i in range(len(self._low_bars) - 1, -1, -1):
            if self._low_bars[i] <= target_bar and self._low_bars[i] >= target_bar - lookback_period:
                found += 1
                if found == instance:
                    return self.count - self._low_bars[i]
        return -1

    # ------------------------------------------------------------------
    #                           reset logic
    # ------------------------------------------------------------------
    cpdef void _reset(self):
        self._highs.clear()
        self._lows.clear()
        self._high_bars.clear()
        self._low_bars.clear()
        self._price_history.clear()

        self.current_high = 0.0
        self.current_low = 0.0
        self.last_swing_price = 0.0
        self.trend_direction = 0
        self.last_swing_idx = -1
        self.count = 0
