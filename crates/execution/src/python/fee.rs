// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
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

//! Python bindings for fee model types.

use nautilus_core::python::to_pyruntime_err;
use nautilus_model::types::Money;
use pyo3::prelude::*;
use rust_decimal::Decimal;

use crate::models::fee::{
    CappedOptionFeeModel, ConfigurableMakerTakerFeeModel, FixedFeeModel, MakerTakerFeeModel,
    PerContractFeeModel, PolymarketFeeModel, TieredNotionalOptionFeeModel,
};

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl FixedFeeModel {
    /// Creates a new `FixedFeeModel` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if `commission` is negative.
    #[new]
    #[pyo3(signature = (commission, change_commission_once=None))]
    fn py_new(commission: Money, change_commission_once: Option<bool>) -> PyResult<Self> {
        Self::new(commission, change_commission_once).map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl MakerTakerFeeModel {
    #[new]
    fn py_new() -> Self {
        Self
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl ConfigurableMakerTakerFeeModel {
    /// Maker/taker fee model with rates configured at construction.
    ///
    /// Identical commission math to `MakerTakerFeeModel` but reads its rates from
    /// the model itself instead of `instrument.maker_fee()` / `instrument.taker_fee()`.
    /// Use when the catalog's baked-in instrument fees are outdated or when fees
    /// need to vary per run independently of instrument metadata.
    #[new]
    #[pyo3(signature = (maker_rate, taker_rate))]
    fn py_new(maker_rate: Decimal, taker_rate: Decimal) -> PyResult<Self> {
        Self::new(maker_rate, taker_rate).map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl PerContractFeeModel {
    /// Creates a new `PerContractFeeModel` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if `commission` is negative.
    #[new]
    fn py_new(commission: Money) -> PyResult<Self> {
        Self::new(commission).map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl CappedOptionFeeModel {
    /// Creates a new `CappedOptionFeeModel` instance.
    #[new]
    #[pyo3(signature = (maker_rate=None, taker_rate=None, cap_rate=None))]
    fn py_new(
        maker_rate: Option<Decimal>,
        taker_rate: Option<Decimal>,
        cap_rate: Option<Decimal>,
    ) -> PyResult<Self> {
        Self::new(maker_rate, taker_rate, cap_rate).map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl TieredNotionalOptionFeeModel {
    /// Creates a new `TieredNotionalOptionFeeModel` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any supplied rate is negative.
    #[new]
    #[pyo3(signature = (maker_rate=None, taker_rate=None))]
    fn py_new(maker_rate: Option<Decimal>, taker_rate: Option<Decimal>) -> PyResult<Self> {
        Self::new(maker_rate, taker_rate).map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl PolymarketFeeModel {
    /// Polymarket-aware fee model implementing the documented `p * (1 - p)` curve.
    ///
    /// Formula: `fee = quantity * fee_rate * p * (1 - p)`, where `fee_rate` is the
    /// effective per-category taker rate read from `instrument.taker_fee()` and
    /// `p` is the fill price in `[0, 1]`. Per Polymarket's fee schedule the rate
    /// is category-dependent (0.07 crypto, 0.04 politics/finance/tech/mentions,
    /// 0.05 economics/culture/weather/general/other, 0.03 sports, 0 geopolitics) —
    /// callers must ensure the instrument's `taker_fee` reflects the documented
    /// effective rate, not gamma's `feeSchedule.rate` which is uniformly 0.25.
    ///
    /// Makers never pay taker fees. When `maker_rebates_enabled` is true (default),
    /// maker fills receive a per-fill credit equal to the documented rebate share
    /// of their fee-equivalent: 20% for crypto markets, 25% for other paying
    /// categories, 0 for fee-free markets. The rebate is inferred from the taker
    /// fee rate (each rebate tier maps to a unique rate or rate group). Returned
    /// as a negative `Money` so the matching engine credits it back to the
    /// account. This is an optimistic upper bound — actual rebates are
    /// distributed daily proportional to the trader's share of maker fee
    /// equivalent against the full market pool, which is not knowable in a
    /// backtest.
    ///
    /// Fees are rounded to 5 decimal places per the docs (smallest charged fee
    /// 0.00001 USDC).
    ///
    /// Reference: <https://docs.polymarket.com/trading/fees>
    /// Reference: <https://docs.polymarket.com/market-makers/maker-rebates>
    #[new]
    #[pyo3(signature = (maker_rebates_enabled=None))]
    fn py_new(maker_rebates_enabled: Option<bool>) -> Self {
        Self::new(maker_rebates_enabled)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}
