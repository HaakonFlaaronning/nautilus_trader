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

import datetime as dt

from nautilus_trader.model.data cimport Bar
from nautilus_trader.model.data cimport BarType
from nautilus_trader.trading.strategy cimport Strategy
from nautilus_trader.trading.config import StrategyConfig


class DemoStrategyCyConfig(StrategyConfig, frozen=True):
    """
    Configuration for ``DemoStrategyCy`` instances.

    Parameters
    ----------
    bar_type_1min : BarType
        The 1-minute bar type for the strategy.

    """
    bar_type_1min: BarType


cdef class DemoStrategyCy(Strategy):
    """
    A Cython demo strategy that counts incoming bars.

    This is a Cython-optimized version for performance testing.
    """

    cdef readonly object instrument_id
    cdef readonly BarType bar_type_1min
    cdef readonly BarType bar_type_5min
    cdef readonly int count_1min_bars
    cdef readonly int count_5min_bars
    cdef object start_time
    cdef object end_time
    cdef readonly object run_time

    def __init__(self, config: DemoStrategyCyConfig):
        super().__init__(config)

        # Extract the trading instrument's ID from the 1-minute bar configuration
        self.instrument_id = config.bar_type_1min.instrument_id

        # Save the 1-minute bar configuration and create a counter to track how many bars we receive
        self.bar_type_1min = config.bar_type_1min
        self.count_1min_bars = 0  # This will increment each time we receive a 1-minute bar

        # Aggregated 5-min bar data
        self.bar_type_5min = BarType.from_str(f"{self.instrument_id}-5-MINUTE-LAST-INTERNAL")
        self.count_5min_bars = 0  # Counter for received 5-minute bars

        # Track when the strategy starts and ends
        self.start_time = None
        self.end_time = None
        self.run_time = None

    cpdef void on_start(self):
        # Start receiving 1-minute bar updates
        self.subscribe_bars(self.bar_type_1min)

    cpdef void on_bar(self, Bar bar):
        # Start the timer when the first bar arrives
        if self.start_time is None:
            self.start_time = dt.datetime.now()

        # Process each bar based on its type
        if bar.bar_type == self.bar_type_1min:  # if 1-minute bar is handled
            self.count_1min_bars += 1
        elif bar.bar_type == self.bar_type_5min:  # if 5-minute bar is handled
            self.count_5min_bars += 1
        else:
            raise Exception(f"Bar type not expected: {bar.bar_type}")

    cpdef void on_stop(self):
        # Save the exact time when strategy ends
        self.end_time = dt.datetime.now()
        self.run_time = self.end_time - self.start_time
        self.log.warning(f"Total run time: {self.run_time}")
