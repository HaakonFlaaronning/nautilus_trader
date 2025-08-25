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

from nautilus_trader.indicators.averages import MovingAverageFactory
from nautilus_trader.indicators.averages import MovingAverageType

from nautilus_trader.core.correctness cimport Condition
from nautilus_trader.core.rust.model cimport PriceType
from nautilus_trader.indicators.base cimport Indicator
from nautilus_trader.model.data cimport Bar
from nautilus_trader.model.data cimport QuoteTick
from nautilus_trader.model.data cimport TradeTick
from nautilus_trader.model.objects cimport Price


cdef class WaveTrendOscillator(Indicator):
    """
    WaveTrend is a fast moving MACD based oscillator that is quick to react on crossovers.
    It calculates the difference between price and exponential moving averages to create
    an oscillator with two lines: a fast line (TCI) and a slow line.

    Parameters
    ----------
    channel_length : int
        The channel length period for the EMA calculations (> 0).
    average_length : int
        The average length period for the TCI calculation (> 0).
    price_type : PriceType
        The specified price type for extracting values from quotes.

    Raises
    ------
    ValueError
        If `channel_length` is not positive (> 0).
    ValueError
        If `average_length` is not positive (> 0).

    References
    ----------
    Based on the WaveTrend oscillator by LazyBear (TradingView)
    """

    def __init__(
        self,
        int channel_length,
        int average_length,
        PriceType price_type=PriceType.LAST,
    ):
        Condition.positive_int(channel_length, "channel_length")
        Condition.positive_int(average_length, "average_length")

        params = [
            channel_length,
            average_length,
        ]
        super().__init__(params=params)

        self.channel_length = channel_length
        self.average_length = average_length
        self.price_type = price_type

        # Create moving averages using EMA for ESA and D, and SMA for WT slow
        self._esa = MovingAverageFactory.create(channel_length, MovingAverageType.EXPONENTIAL)
        self._d_ema = MovingAverageFactory.create(channel_length, MovingAverageType.EXPONENTIAL)
        self._tci_ema = MovingAverageFactory.create(average_length, MovingAverageType.EXPONENTIAL)
        self._wt_slow = MovingAverageFactory.create(4, MovingAverageType.SIMPLE)

        self.wt_fast = 0
        self.wt_slow = 0

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
        cdef double price_val = Price.raw_to_f64_c(price._mem.raw)
        # For quotes, we use the price as high, low, and close
        self.update_raw(price_val, price_val, price_val)

    cpdef void handle_trade_tick(self, TradeTick tick):
        """
        Update the indicator with the given trade tick.

        Parameters
        ----------
        tick : TradeTick
            The update tick to handle.

        """
        Condition.not_none(tick, "tick")

        cdef double price = Price.raw_to_f64_c(tick._mem.price.raw)
        # For trades, we use the price as high, low, and close
        self.update_raw(price, price, price)

    cpdef void handle_bar(self, Bar bar):
        """
        Update the indicator with the given bar.

        Parameters
        ----------
        bar : Bar
            The update bar to handle.

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
        # Check if first input
        if not self.has_inputs:
            self._set_has_inputs(True)

        # Calculate typical price
        cdef double ap = (high + low + close) / 3.0

        # Update ESA (EMA of typical price)
        self._esa.update_raw(ap)

        # Calculate absolute difference and update d_ema (EMA of absolute difference)
        cdef double d_absolute = abs(ap - self._esa.value)
        self._d_ema.update_raw(d_absolute)

        # Calculate Channel Index (CI)
        cdef double ci = 0
        if self._d_ema.value != 0:
            ci = (ap - self._esa.value) / (0.015 * self._d_ema.value)

        # Update TCI (EMA of CI) - this is the fast WaveTrend line
        self._tci_ema.update_raw(ci)
        self.wt_fast = self._tci_ema.value

        # Update slow WaveTrend line (SMA of fast line)
        self._wt_slow.update_raw(self.wt_fast)
        self.wt_slow = self._wt_slow.value

        # Initialization logic
        if not self.initialized:
            if (self._esa.initialized and
                self._d_ema.initialized and
                self._tci_ema.initialized and
                self._wt_slow.initialized):
                self._set_initialized(True)

    cpdef void _reset(self):
        self._esa.reset()
        self._d_ema.reset()
        self._tci_ema.reset()
        self._wt_slow.reset()
        self.wt_fast = 0
        self.wt_slow = 0
