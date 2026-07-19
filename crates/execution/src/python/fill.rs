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

//! Python bindings for fill model types.

use nautilus_core::python::to_pyruntime_err;
use pyo3::prelude::*;

use crate::models::fill::{
    BestPriceFillModel, CompetitionAwareFillModel, DefaultFillModel, FixedTickSlippageFillModel,
    LimitOrderPartialFillModel, MarketHoursFillModel, OneTickSlippageFillModel,
    PolymarketFixedSlippageFillModel, ProbabilisticFillModel, SizeAwareFillModel,
    ThreeTierFillModel, TwoTierFillModel, VolumeSensitiveFillModel,
};

macro_rules! impl_fill_model_pymethods {
    ($type:ty) => {
        #[pymethods]
        #[pyo3_stub_gen::derive::gen_stub_pymethods]
        impl $type {
            #[new]
            #[pyo3(signature = (prob_fill_on_limit=1.0, prob_slippage=0.0, random_seed=None))]
            fn py_new(
                prob_fill_on_limit: f64,
                prob_slippage: f64,
                random_seed: Option<u64>,
            ) -> PyResult<Self> {
                Self::new(prob_fill_on_limit, prob_slippage, random_seed).map_err(to_pyruntime_err)
            }

            fn __repr__(&self) -> String {
                format!("{self:?}")
            }
        }
    };
}

impl_fill_model_pymethods!(DefaultFillModel);
impl_fill_model_pymethods!(BestPriceFillModel);
impl_fill_model_pymethods!(OneTickSlippageFillModel);
impl_fill_model_pymethods!(ProbabilisticFillModel);
impl_fill_model_pymethods!(TwoTierFillModel);
impl_fill_model_pymethods!(ThreeTierFillModel);
impl_fill_model_pymethods!(LimitOrderPartialFillModel);
impl_fill_model_pymethods!(SizeAwareFillModel);
impl_fill_model_pymethods!(VolumeSensitiveFillModel);
impl_fill_model_pymethods!(MarketHoursFillModel);

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl FixedTickSlippageFillModel {
    /// Fill model that forces a configurable number of ticks of slippage for all
    /// orders. Aggressive buys fill at `best_ask + slippage_ticks * tick`;
    /// aggressive sells fill at `best_bid - slippage_ticks * tick`, using each
    /// instrument's own price increment at fill time.
    #[new]
    #[pyo3(signature = (slippage_ticks=1, prob_fill_on_limit=1.0, prob_slippage=0.0, random_seed=None))]
    fn py_new(
        slippage_ticks: u32,
        prob_fill_on_limit: f64,
        prob_slippage: f64,
        random_seed: Option<u64>,
    ) -> PyResult<Self> {
        Self::new(slippage_ticks, prob_fill_on_limit, prob_slippage, random_seed)
            .map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl PolymarketFixedSlippageFillModel {
    /// Fill model that forces a configurable fixed slippage on every fill, with
    /// optional clamping to the Polymarket binary-option `[tick, 1 - tick]`
    /// probability domain.
    ///
    /// Builds a synthetic L2 book with unlimited liquidity sitting `slippage`
    /// price units away from the matching engine's transient best bid/ask
    /// (which, on Polymarket trade-tick replay, equals the last traded price).
    /// Aggressive buys fill at `best_ask + slippage`; aggressive sells fill at
    /// `best_bid - slippage`. When `clamp_to_probability_domain` is true (default
    /// for prediction markets), the shifted prices are clamped into Polymarket's
    /// `[tick, 1 - tick]` domain so post-slippage prices remain valid.
    ///
    /// Differs from `OneTickSlippageFillModel` in that the slippage amount is
    /// configurable in absolute price units rather than hard-coded to one tick.
    /// Deterministic — `is_slipped()` always returns true. The right fit for
    /// markets like Polymarket where a uniform fixed slippage applies to every
    /// fill.
    #[new]
    #[pyo3(signature = (slippage, clamp_to_probability_domain=None))]
    fn py_new(slippage: f64, clamp_to_probability_domain: Option<bool>) -> PyResult<Self> {
        Self::new(slippage, clamp_to_probability_domain).map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[pymethods]
#[pyo3_stub_gen::derive::gen_stub_pymethods]
impl CompetitionAwareFillModel {
    /// Fill model that reduces available liquidity by a factor to simulate market competition.
    #[new]
    #[pyo3(signature = (
        prob_fill_on_limit=1.0,
        prob_slippage=0.0,
        random_seed=None,
        liquidity_factor=0.3,
    ))]
    fn py_new(
        prob_fill_on_limit: f64,
        prob_slippage: f64,
        random_seed: Option<u64>,
        liquidity_factor: f64,
    ) -> PyResult<Self> {
        Self::new(
            prob_fill_on_limit,
            prob_slippage,
            random_seed,
            liquidity_factor,
        )
        .map_err(to_pyruntime_err)
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}
