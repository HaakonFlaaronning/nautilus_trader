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

from nautilus_trader.indicators.averages cimport MovingAverage


cdef class ZeroLagExponentialMovingAverage(MovingAverage):
    cdef readonly double alpha
    """The moving average alpha value.\n\n:returns: `double`"""
    cdef readonly int lag
    """The lag period used for zero-lag calculation.\n\n:returns: `int`"""
    cdef object _inputs
    """The input values buffer for lag calculation.\n\n:returns: `deque`"""
