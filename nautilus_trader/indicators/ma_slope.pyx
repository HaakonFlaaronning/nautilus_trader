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
from collections import deque

from nautilus_trader.indicators.average.ma_factory import MovingAverageFactory
from nautilus_trader.indicators.average.moving_average import MovingAverageType
from nautilus_trader.core.correctness cimport Condition
from nautilus_trader.core.rust.model cimport PriceType
from nautilus_trader.indicators.base.indicator cimport Indicator
from nautilus_trader.model.data cimport Bar
from nautilus_trader.model.data cimport QuoteTick
from nautilus_trader.model.data cimport TradeTick
from nautilus_trader.model.objects cimport Price


cdef class MASlope(Indicator):
    """
    An indicator which calculates the difference between two moving averages.
    Different moving average types can be selected for the inner calculation.

    Parameters
    ----------
    fast_period : int
        The period for the fast moving average (> 0).
    slow_period : int
        The period for the slow moving average (> 0 & > fast_sma).
    ma_period : int
        The period for the moving average used to calculate the slope (> 0).
    ma_type : MovingAverageType
        The moving average type for the calculations.
    price_type : PriceType
        The specified price type for extracting values from quotes.

    Raises
    ------
    ValueError
        If `fast_period` is not positive (> 0).
    ValueError
        If `slow_period` is not positive (> 0).
    ValueError
        If `fast_period` is not < `slow_period`.
    """

    def __init__(
        self,
        int ma_period,
        int start_bars_ago,
        int end_bars_ago=0,
        ma_type not None: MovingAverageType=MovingAverageType.SIMPLE,
        PriceType price_type=PriceType.LAST,
    ):
        Condition.positive_int(ma_period, "ma_period")

        params=[
            start_bars_ago,
            end_bars_ago,
            ma_type.name,
        ]
        super().__init__(params=params)

        self.start_bars_ago = start_bars_ago if start_bars_ago >= 0 else ma_period + start_bars_ago
        self.end_bars_ago = end_bars_ago if end_bars_ago >= 0 else ma_period + end_bars_ago
        Condition.is_true(self.start_bars_ago > self.end_bars_ago, "end_bars_ago was > start_bars_ago")

        self._ma = MovingAverageFactory.create(ma_period, ma_type)
        self._ma_prices = deque(maxlen=ma_period)
        self.price_type = price_type
        self.value = 0


    cpdef void handle_quote_tick(self, QuoteTick tick):
        """
        Update the indicator with the given quote tick.

        Parameters
        ----------
        tick : QuoteTick
            The update tick to handle.

        """
        Condition.not_none(tick, "tick")

        cdef Price price = tick.extract_price(self.price_type)
        self.update_raw(Price.raw_to_f64_c(price._mem.raw))

    cpdef void handle_trade_tick(self, TradeTick tick):
        """
        Update the indicator with the given trade tick.

        Parameters
        ----------
        tick : TradeTick
            The update tick to handle.

        """
        Condition.not_none(tick, "tick")

        self.update_raw(Price.raw_to_f64_c(tick._mem.price.raw))

    cpdef void handle_bar(self, Bar bar):
        """
        Update the indicator with the given bar.

        Parameters
        ----------
        bar : Bar
            The update bar.

        """
        Condition.not_none(bar, "bar")

        self.update_raw(bar.close.as_double())

    cpdef void update_raw(self, double close):
        """
        Update the indicator with the given close price.

        Parameters
        ----------
        close : double
            The close price.

        """
        self._ma.update_raw(close)
        self._ma_prices.appendleft(self._ma.value)

        start_idx = self._clamp_index(self.start_bars_ago, len(self._ma_prices))
        end_idx = self._clamp_index(self.end_bars_ago, len(self._ma_prices))

        if start_idx == end_idx:
            self.value = 0
        else:
            self.value = (self._ma_prices[end_idx] - self._ma_prices[start_idx]) / (start_idx - end_idx)

        # Initialization logic
        if not self.initialized:
            self._set_has_inputs(True)
            if self._ma.initialized:
                self._set_initialized(True)

    cpdef void _reset(self):
        self._ma.reset()
        self.value = 0

    cpdef int _clamp_index(self, int idx, int n):
        """
        Clamp the index to the range of the moving average.

        Parameters
        ----------
        idx : int
            The index to clamp.
        n : int
            The number of bars in the moving average.

        Returns
        -------
        int
            The clamped index.
        """
        if idx < 0:
            idx = 0
        elif idx >= n:
            idx = n - 1
        return idx
