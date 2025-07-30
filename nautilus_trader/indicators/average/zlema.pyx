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
import math
from collections import deque

from nautilus_trader.core.correctness cimport Condition
from nautilus_trader.core.rust.model cimport PriceType
from nautilus_trader.indicators.average.moving_average cimport MovingAverage
from nautilus_trader.model.data cimport Bar
from nautilus_trader.model.data cimport QuoteTick
from nautilus_trader.model.data cimport TradeTick
from nautilus_trader.model.objects cimport Price


cdef class ZeroLagExponentialMovingAverage(MovingAverage):
    """
    An indicator which calculates a zero-lag exponential moving average across a
    rolling window.

    The ZLEMA attempts to eliminate the lag inherent in moving averages by
    incorporating a momentum term based on the rate of change.

    Parameters
    ----------
    period : int
        The rolling window period for the indicator (> 0).
    price_type : PriceType
        The specified price type for extracting values from quotes.

    Raises
    ------
    ValueError
        If `period` is not positive (> 0).
    """

    def __init__(self, int period, PriceType price_type=PriceType.LAST):
        Condition.positive_int(period, "period")
        super().__init__(period, params=[period], price_type=price_type)

        self.alpha = 2.0 / (period + 1.0)
        self.lag = math.ceil((period - 1) / 2.0)
        # We need to store lag+1 values to have access to the value at index 0 when we have lag+1 elements
        self._inputs = deque(maxlen=self.lag + 1)
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
            The update bar to handle.

        """
        Condition.not_none(bar, "bar")

        self.update_raw(bar.close.as_double())

    cpdef void update_raw(self, double value):
        """
        Update the indicator with the given raw value.

        Parameters
        ----------
        value : double
            The update value.

        """
        # Store the input value
        self._inputs.append(value)

        # Check if this is the initial input or if we don't have enough history for lag
        if not self.has_inputs or len(self._inputs) <= self.lag:
            self.value = value
        else:
            lagged_value = self._inputs[0]  # Oldest value in deque
            adjusted_value = 2.0 * value - lagged_value

            # Calculate the zero-lag EMA
            self.value = self.alpha * adjusted_value + ((1.0 - self.alpha) * self.value)

        self._increment_count()

    cpdef void _reset_ma(self):
        """
        Reset the moving average specific state.
        """
        self._inputs.clear()
