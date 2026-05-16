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
from nautilus_trader.indicators.base cimport Indicator


cdef class ZigZag(Indicator):
    cdef list _highs
    cdef list _lows
    cdef list _high_bars
    cdef list _low_bars
    cdef list _price_history

    cdef readonly double deviation_value
    """The deviation value for the ZigZag calculation.\n\n:returns: `double`"""
    cdef readonly bint use_point_deviation_type
    """Whether to use points deviation type instead of percent.\n\n:returns: `bool`"""
    cdef readonly bint use_high_low
    """Whether to use high/low prices instead of close prices.\n\n:returns: `bool`"""
    cdef readonly double current_high
    """The current ZigZag high value.\n\n:returns: `double`"""
    cdef readonly double current_low
    """The current ZigZag low value.\n\n:returns: `double`"""
    cdef readonly double last_swing_price
    """The last swing price.\n\n:returns: `double`"""
    cdef readonly int trend_direction
    """The current trend direction (1=up, -1=down, 0=init).\n\n:returns: `int`"""
    cdef readonly int last_swing_idx
    """The last swing index.\n\n:returns: `int`"""
    cdef readonly int count
    """The number of bars processed.\n\n:returns: `int`"""

    cpdef void update_raw(self, double high, double low, double close)
    cpdef list get_highs(self)
    cpdef list get_lows(self)
    cpdef list get_high_bars(self)
    cpdef list get_low_bars(self)
    cpdef int high_bar(self, int bars_ago, int instance, int lookback_period)
    cpdef int low_bar(self, int bars_ago, int instance, int lookback_period)
