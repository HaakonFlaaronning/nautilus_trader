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

from nautilus_trader.core.rust.model cimport PriceType
from nautilus_trader.indicators.averages cimport MovingAverage
from nautilus_trader.indicators.base cimport Indicator


cdef class MASlope(Indicator):
    cdef MovingAverage _ma
    cdef object _ma_prices

    cdef readonly PriceType price_type
    """The specified price type for extracting values from quotes.\n\n:returns: `PriceType`"""
    cdef readonly int start_bars_ago
    """The starting point for slope calculation (bars ago).\n\n:returns: `int`"""
    cdef readonly int end_bars_ago
    """The ending point for slope calculation (bars ago).\n\n:returns: `int`"""
    cdef readonly double value
    """The current slope value.\n\n:returns: `double`"""

    cpdef void update_raw(self, double close)
    cpdef int _clamp_index(self, int idx, int n)
