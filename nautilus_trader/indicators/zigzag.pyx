# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

from nautilus_trader.core.correctness cimport Condition
from nautilus_trader.indicators.base.indicator cimport Indicator
from nautilus_trader.model.data cimport Bar


cdef class ZigZag(Indicator):
    """
    The ZigZag indicator shows trend lines filtering out changes below a defined level.

    This indicator identifies significant price reversals by filtering out price movements
    that don't meet a minimum deviation threshold. It connects swing highs and swing lows
    that exceed the specified deviation, creating a zigzag pattern that highlights the
    main trend movements while filtering out minor fluctuations.

    Parameters
    ----------
    deviation_value : double
        The minimum deviation required to register a new swing point.
    use_point_deviation_type : bool, default True
        Whether to use points deviation type (True) or percent deviation type (False).
    use_high_low : bool, default True
        Whether to use high/low prices (True) or close prices (False) for calculations.

    Raises
    ------
    ValueError
        If `deviation_value` is not positive (> 0).
    """

    def __init__(
        self,
        double deviation_value,
        bint use_point_deviation_type=True,
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

        # Initialize internal state
        self._highs = []
        self._lows = []
        self._high_bars = []
        self._low_bars = []

        # Store last few bars for swing detection
        self._price_history = []

        self.current_high = 0.0
        self.current_low = 0.0
        self.last_swing_price = 0.0
        self.trend_direction = 0  # 1 = up trend, -1 = down trend, 0 = init
        self.last_swing_idx = -1
        self.count = 0  # Track number of bars processed

    cpdef void handle_bar(self, Bar bar):
        """
        Update the indicator with the given bar.

        Parameters
        ----------
        bar : Bar
            The update bar.
        """
        Condition.not_none(bar, "bar")

        self.update_raw(
            bar.high.as_double(),
            bar.low.as_double(),
            bar.close.as_double(),
        )

    cpdef void update_raw(self, double high, double low, double close):
        """
        Update the indicator with the given raw values.

        Parameters
        ----------
        high : double
            The high price.
        low : double
            The low price.
        close : double
            The close price.
        """
        # Increment bar count
        self.count += 1

        # Store current bar data
        cdef dict current_bar = {
            'high': high,
            'low': low,
            'close': close,
            'bar_index': self.count
        }
        self._price_history.append(current_bar)

        # Keep only last 3 bars for swing detection
        if len(self._price_history) > 3:
            self._price_history.pop(0)

        # Set has inputs after first bar
        if not self.has_inputs:
            self._set_has_inputs(True)

        # Need at least 3 bars for swing calculation
        if len(self._price_history) < 3:
            return

        # Initialize last swing price on first valid update
        if self.last_swing_price == 0.0:
            self.last_swing_price = close

        # Get price series to use
        cdef list high_series = []
        cdef list low_series = []

        for i in range(len(self._price_history)):
            if self.use_high_low:
                high_series.append(self._price_history[i]['high'])
                low_series.append(self._price_history[i]['low'])
            else:
                high_series.append(self._price_history[i]['close'])
                low_series.append(self._price_history[i]['close'])

        # Check for swing high/low on the middle bar (index 1)
        cdef bint is_swing_high = (high_series[1] >= high_series[0] and
                                   high_series[1] >= high_series[2])
        cdef bint is_swing_low = (low_series[1] <= low_series[0] and
                                  low_series[1] <= low_series[2])

        # Calculate deviation thresholds
        cdef double high_threshold, low_threshold
        if self.use_point_deviation_type:
            high_threshold = self.last_swing_price + self.deviation_value
            low_threshold = self.last_swing_price - self.deviation_value
        else:  # PERCENT
            high_threshold = self.last_swing_price * (1.0 + self.deviation_value / 100.0)
            low_threshold = self.last_swing_price * (1.0 - self.deviation_value / 100.0)

        cdef bint is_over_high_deviation = high_series[1] > high_threshold
        cdef bint is_over_low_deviation = low_series[1] < low_threshold

        cdef double save_value = 0.0
        cdef bint add_high = False
        cdef bint add_low = False
        cdef bint update_high = False
        cdef bint update_low = False

        # Skip if no swing detected
        if not is_swing_high and not is_swing_low:
            return

        # Check conditions for new swing points
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
        elif self.trend_direction == -1 and is_swing_low and low_series[1] < self.last_swing_price:
            save_value = low_series[1]
            update_low = True

        # Process swing point updates
        if add_high or add_low or update_high or update_low:
            if update_high and len(self._highs) > 0:
                # Update the last high point
                self._highs[-1] = save_value
                self._high_bars[-1] = self._price_history[1]['bar_index']
            elif update_low and len(self._lows) > 0:
                # Update the last low point
                self._lows[-1] = save_value
                self._low_bars[-1] = self._price_history[1]['bar_index']

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

            # Mark as initialized after processing first valid swing
            if not self.initialized:
                self._set_initialized(True)

    cpdef list get_highs(self):
        """
        Return the list of ZigZag high values.

        Returns
        -------
        list[double]
            The ZigZag high values.
        """
        return self._highs.copy()

    cpdef list get_lows(self):
        """
        Return the list of ZigZag low values.

        Returns
        -------
        list[double]
            The ZigZag low values.
        """
        return self._lows.copy()

    cpdef list get_high_bars(self):
        """
        Return the list of bar indices where ZigZag highs occurred.

        Returns
        -------
        list[int]
            The bar indices of ZigZag highs.
        """
        return self._high_bars.copy()

    cpdef list get_low_bars(self):
        """
        Return the list of bar indices where ZigZag lows occurred.

        Returns
        -------
        list[int]
            The bar indices of ZigZag lows.
        """
        return self._low_bars.copy()

    cpdef int high_bar(self, int bars_ago, int instance, int lookback_period):
        """
        Returns the number of bars ago a ZigZag high occurred.

        Parameters
        ----------
        bars_ago : int
            The number of bars ago to start looking from.
        instance : int
            The instance of the high to find (1-based).
        lookback_period : int
            The maximum number of bars to look back.

        Returns
        -------
        int
            The number of bars ago the high occurred, or -1 if not found.
        """
        if instance < 1:
            return -1
        if bars_ago < 0:
            return -1

        cdef int target_bar = self.count - bars_ago - 1
        cdef int found_instance = 0

        for i in range(len(self._high_bars) - 1, -1, -1):
            if self._high_bars[i] <= target_bar and self._high_bars[i] >= target_bar - lookback_period:
                found_instance += 1
                if found_instance == instance:
                    return self.count - self._high_bars[i]

        return -1

    cpdef int low_bar(self, int bars_ago, int instance, int lookback_period):
        """
        Returns the number of bars ago a ZigZag low occurred.

        Parameters
        ----------
        bars_ago : int
            The number of bars ago to start looking from.
        instance : int
            The instance of the low to find (1-based).
        lookback_period : int
            The maximum number of bars to look back.

        Returns
        -------
        int
            The number of bars ago the low occurred, or -1 if not found.
        """
        if instance < 1:
            return -1
        if bars_ago < 0:
            return -1

        cdef int target_bar = self.count - bars_ago - 1
        cdef int found_instance = 0

        for i in range(len(self._low_bars) - 1, -1, -1):
            if self._low_bars[i] <= target_bar and self._low_bars[i] >= target_bar - lookback_period:
                found_instance += 1
                if found_instance == instance:
                    return self.count - self._low_bars[i]

        return -1

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
