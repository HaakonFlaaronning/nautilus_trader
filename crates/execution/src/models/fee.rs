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

use std::fmt::Debug;

use nautilus_model::{
    enums::LiquiditySide,
    instruments::{Instrument, InstrumentAny},
    orders::{Order, OrderAny},
    types::{Currency, Money, Price, Quantity},
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub trait FeeModel {
    /// Calculates commission for a fill.
    ///
    /// # Errors
    ///
    /// Returns an error if commission calculation fails.
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money>;

    /// Calculates commission for a fill with additional pricing context.
    ///
    /// # Errors
    ///
    /// Returns an error if commission calculation fails.
    fn get_commission_with_context(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
        _underlying_px: Option<Price>,
    ) -> anyhow::Result<Money> {
        self.get_commission(order, fill_quantity, fill_px, instrument)
    }
}

#[derive(Clone, Debug)]
pub enum FeeModelAny {
    Fixed(FixedFeeModel),
    MakerTaker(MakerTakerFeeModel),
    PerContract(PerContractFeeModel),
    CappedOption(CappedOptionFeeModel),
    TieredNotionalOption(TieredNotionalOptionFeeModel),
    Polymarket(PolymarketFeeModel),
    ConfigurableMakerTaker(ConfigurableMakerTakerFeeModel),
}

impl FeeModel for FeeModelAny {
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        match self {
            Self::Fixed(model) => model.get_commission(order, fill_quantity, fill_px, instrument),
            Self::MakerTaker(model) => {
                model.get_commission(order, fill_quantity, fill_px, instrument)
            }
            Self::PerContract(model) => {
                model.get_commission(order, fill_quantity, fill_px, instrument)
            }
            Self::CappedOption(model) => {
                model.get_commission(order, fill_quantity, fill_px, instrument)
            }
            Self::TieredNotionalOption(model) => {
                model.get_commission(order, fill_quantity, fill_px, instrument)
            }
            Self::Polymarket(model) => {
                model.get_commission(order, fill_quantity, fill_px, instrument)
            }
            Self::ConfigurableMakerTaker(model) => {
                model.get_commission(order, fill_quantity, fill_px, instrument)
            }
        }
    }

    fn get_commission_with_context(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
        underlying_px: Option<Price>,
    ) -> anyhow::Result<Money> {
        match self {
            Self::Fixed(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
            Self::MakerTaker(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
            Self::PerContract(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
            Self::CappedOption(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
            Self::TieredNotionalOption(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
            Self::Polymarket(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
            Self::ConfigurableMakerTaker(model) => model.get_commission_with_context(
                order,
                fill_quantity,
                fill_px,
                instrument,
                underlying_px,
            ),
        }
    }
}

impl Default for FeeModelAny {
    fn default() -> Self {
        Self::MakerTaker(MakerTakerFeeModel)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct FixedFeeModel {
    commission: Money,
    zero_commission: Money,
    change_commission_once: bool,
}

impl FixedFeeModel {
    /// Creates a new [`FixedFeeModel`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if `commission` is negative.
    pub fn new(commission: Money, change_commission_once: Option<bool>) -> anyhow::Result<Self> {
        if commission.raw < 0 {
            anyhow::bail!("Commission must be greater than or equal to zero")
        }
        let zero_commission = Money::zero(commission.currency);
        Ok(Self {
            commission,
            zero_commission,
            change_commission_once: change_commission_once.unwrap_or(true),
        })
    }
}

impl FeeModel for FixedFeeModel {
    fn get_commission(
        &self,
        order: &OrderAny,
        _fill_quantity: Quantity,
        _fill_px: Price,
        _instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        if !self.change_commission_once || order.filled_qty().is_zero() {
            Ok(self.commission)
        } else {
            Ok(self.zero_commission)
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct PerContractFeeModel {
    commission: Money,
}

impl PerContractFeeModel {
    /// Creates a new [`PerContractFeeModel`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if `commission` is negative.
    pub fn new(commission: Money) -> anyhow::Result<Self> {
        if commission.raw < 0 {
            anyhow::bail!("Commission must be greater than or equal to zero")
        }
        Ok(Self { commission })
    }
}

impl FeeModel for PerContractFeeModel {
    fn get_commission(
        &self,
        _order: &OrderAny,
        fill_quantity: Quantity,
        _fill_px: Price,
        _instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        let total = self.commission.as_decimal() * fill_quantity.as_decimal();
        Money::from_decimal(total, self.commission.currency).map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct MakerTakerFeeModel;

impl FeeModel for MakerTakerFeeModel {
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        let notional = instrument.calculate_notional_value(fill_quantity, fill_px, Some(false));
        let commission = match order.liquidity_side() {
            Some(LiquiditySide::Maker) => notional * instrument.maker_fee(),
            Some(LiquiditySide::Taker) => notional * instrument.taker_fee(),
            Some(LiquiditySide::NoLiquiditySide) | None => anyhow::bail!("Liquidity side not set"),
        };

        if instrument.is_inverse() {
            Money::from_decimal(commission, instrument.base_currency().unwrap()).map_err(Into::into)
        } else {
            Money::from_decimal(commission, instrument.quote_currency()).map_err(Into::into)
        }
    }
}

/// Maker/taker fee model with rates configured at construction.
///
/// Identical commission math to [`MakerTakerFeeModel`] but reads its rates from
/// the model itself instead of `instrument.maker_fee()` / `instrument.taker_fee()`.
/// Use when the catalog's baked-in instrument fees are outdated or when fees
/// need to vary per run independently of instrument metadata (e.g. simulating
/// different exchange tiers).
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct ConfigurableMakerTakerFeeModel {
    maker_rate: Decimal,
    taker_rate: Decimal,
}

impl ConfigurableMakerTakerFeeModel {
    /// Creates a new [`ConfigurableMakerTakerFeeModel`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if either rate is negative.
    pub fn new(maker_rate: Decimal, taker_rate: Decimal) -> anyhow::Result<Self> {
        check_fee_rate(Some(maker_rate), "maker_rate")?;
        check_fee_rate(Some(taker_rate), "taker_rate")?;
        Ok(Self {
            maker_rate,
            taker_rate,
        })
    }
}

impl FeeModel for ConfigurableMakerTakerFeeModel {
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        let notional = instrument.calculate_notional_value(fill_quantity, fill_px, Some(false));
        let commission = match order.liquidity_side() {
            Some(LiquiditySide::Maker) => notional * self.maker_rate,
            Some(LiquiditySide::Taker) => notional * self.taker_rate,
            Some(LiquiditySide::NoLiquiditySide) | None => anyhow::bail!("Liquidity side not set"),
        };

        if instrument.is_inverse() {
            Money::from_decimal(commission, instrument.base_currency().unwrap()).map_err(Into::into)
        } else {
            Money::from_decimal(commission, instrument.quote_currency()).map_err(Into::into)
        }
    }
}

#[derive(Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct CappedOptionFeeModel {
    maker_rate: Option<Decimal>,
    taker_rate: Option<Decimal>,
    cap: Decimal,
}

impl Debug for CappedOptionFeeModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(CappedOptionFeeModel))
            .field("maker_rate", &self.maker_rate)
            .field("taker_rate", &self.taker_rate)
            .field("cap_rate", &self.cap)
            .finish()
    }
}

impl CappedOptionFeeModel {
    /// Creates a new [`CappedOptionFeeModel`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any supplied rate is negative.
    pub fn new(
        maker_rate: Option<Decimal>,
        taker_rate: Option<Decimal>,
        cap_rate: Option<Decimal>,
    ) -> anyhow::Result<Self> {
        check_fee_rate(maker_rate, "maker_rate")?;
        check_fee_rate(taker_rate, "taker_rate")?;

        let cap_rate = cap_rate.unwrap_or(dec!(0.125));
        check_fee_rate(Some(cap_rate), "cap_rate")?;

        Ok(Self {
            maker_rate,
            taker_rate,
            cap: cap_rate,
        })
    }
}

impl Default for CappedOptionFeeModel {
    fn default() -> Self {
        Self::new(None, None, None).unwrap()
    }
}

impl FeeModel for CappedOptionFeeModel {
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        self.get_commission_with_context(order, fill_quantity, fill_px, instrument, None)
    }

    fn get_commission_with_context(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
        underlying_px: Option<Price>,
    ) -> anyhow::Result<Money> {
        check_option_instrument(instrument, "CappedOptionFeeModel")?;
        let rate = option_fee_rate(order, instrument, self.maker_rate, self.taker_rate)?;
        let multiplier = instrument.multiplier().as_decimal();
        let rate_fee = if instrument.is_inverse() {
            rate
        } else {
            let underlying_px =
                underlying_px.ok_or_else(|| anyhow::anyhow!("Underlying price is required"))?;
            rate * underlying_px.as_decimal()
        };
        let cap_fee = self.cap * fill_px.as_decimal();
        let fee_per_contract = rate_fee.min(cap_fee) * multiplier;
        let total = fee_per_contract * fill_quantity.as_decimal();
        Money::from_decimal(total, commission_currency(instrument)).map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct TieredNotionalOptionFeeModel {
    maker_rate: Option<Decimal>,
    taker_rate: Option<Decimal>,
}

impl TieredNotionalOptionFeeModel {
    /// Creates a new [`TieredNotionalOptionFeeModel`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any supplied rate is negative.
    pub fn new(maker_rate: Option<Decimal>, taker_rate: Option<Decimal>) -> anyhow::Result<Self> {
        check_fee_rate(maker_rate, "maker_rate")?;
        check_fee_rate(taker_rate, "taker_rate")?;

        Ok(Self {
            maker_rate,
            taker_rate,
        })
    }
}

impl Default for TieredNotionalOptionFeeModel {
    fn default() -> Self {
        Self::new(None, None).unwrap()
    }
}

impl FeeModel for TieredNotionalOptionFeeModel {
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        check_option_instrument(instrument, "TieredNotionalOptionFeeModel")?;
        let rate = option_fee_rate(order, instrument, self.maker_rate, self.taker_rate)?;
        let notional = instrument.calculate_notional_value(fill_quantity, fill_px, Some(false));
        let total = notional.as_decimal() * rate;
        Money::from_decimal(total, notional.currency).map_err(Into::into)
    }
}

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
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(
        module = "nautilus_trader.core.nautilus_pyo3.execution",
        from_py_object
    )
)]
#[cfg_attr(
    feature = "python",
    pyo3_stub_gen::derive::gen_stub_pyclass(module = "nautilus_trader.execution")
)]
pub struct PolymarketFeeModel {
    maker_rebates_enabled: bool,
}

impl PolymarketFeeModel {
    /// Creates a new [`PolymarketFeeModel`] instance.
    #[must_use]
    pub fn new(maker_rebates_enabled: Option<bool>) -> Self {
        Self {
            maker_rebates_enabled: maker_rebates_enabled.unwrap_or(true),
        }
    }
}

impl Default for PolymarketFeeModel {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FeeModel for PolymarketFeeModel {
    fn get_commission(
        &self,
        order: &OrderAny,
        fill_quantity: Quantity,
        fill_px: Price,
        instrument: &InstrumentAny,
    ) -> anyhow::Result<Money> {
        let quote_currency = instrument.quote_currency();
        let taker_fee = instrument.taker_fee();
        if taker_fee <= Decimal::ZERO {
            return Ok(Money::zero(quote_currency));
        }

        let qty = fill_quantity.as_decimal();
        let p = fill_px.as_decimal();
        let fee_equivalent = qty * taker_fee * p * (dec!(1) - p);

        let amount = match order.liquidity_side() {
            Some(LiquiditySide::Maker) => {
                if !self.maker_rebates_enabled {
                    return Ok(Money::zero(quote_currency));
                }
                let rebate_share = polymarket_rebate_share(taker_fee);
                if rebate_share.is_zero() {
                    return Ok(Money::zero(quote_currency));
                }
                -(fee_equivalent * rebate_share).round_dp(5)
            }
            // Taker / NoLiquiditySide / unset: charge taker fee.
            _ => fee_equivalent.round_dp(5),
        };

        Money::from_decimal(amount, quote_currency).map_err(Into::into)
    }
}

/// Maker rebate share for a Polymarket market, derived from the documented
/// taker fee rate. Crypto markets (rate 0.07) pay 20% of the rebate pool;
/// other paying categories (rates 0.03 / 0.04 / 0.05) pay 25%. Fee-free
/// markets (rate 0) and unknown rates return 0.
///
/// Reference: <https://docs.polymarket.com/market-makers/maker-rebates>
fn polymarket_rebate_share(fee_rate: Decimal) -> Decimal {
    let rate = fee_rate.normalize();
    if rate == dec!(0.07) {
        dec!(0.20)
    } else if rate == dec!(0.03) || rate == dec!(0.04) || rate == dec!(0.05) {
        dec!(0.25)
    } else {
        dec!(0)
    }
}

fn option_fee_rate(
    order: &OrderAny,
    instrument: &InstrumentAny,
    maker_rate: Option<Decimal>,
    taker_rate: Option<Decimal>,
) -> anyhow::Result<Decimal> {
    let rate = match order.liquidity_side() {
        Some(LiquiditySide::Maker) => maker_rate.unwrap_or_else(|| instrument.maker_fee()),
        Some(LiquiditySide::Taker) => taker_rate.unwrap_or_else(|| instrument.taker_fee()),
        Some(LiquiditySide::NoLiquiditySide) | None => anyhow::bail!("Liquidity side not set"),
    };
    check_fee_rate(Some(rate), "fee_rate")?;
    Ok(rate)
}

fn check_fee_rate(rate: Option<Decimal>, name: &str) -> anyhow::Result<()> {
    if rate.is_some_and(|rate| rate < Decimal::ZERO) {
        anyhow::bail!("`{name}` must be greater than or equal to zero");
    }
    Ok(())
}

fn check_option_instrument(instrument: &InstrumentAny, model_name: &str) -> anyhow::Result<()> {
    if !matches!(
        instrument,
        InstrumentAny::CryptoOption(_) | InstrumentAny::OptionContract(_)
    ) {
        anyhow::bail!("{model_name} requires an option instrument");
    }
    Ok(())
}

fn commission_currency(instrument: &InstrumentAny) -> Currency {
    if instrument.is_inverse() {
        instrument.settlement_currency()
    } else {
        instrument.quote_currency()
    }
}

#[cfg(test)]
mod tests {
    use nautilus_model::{
        enums::{LiquiditySide, OrderSide, OrderType},
        instruments::{
            CryptoOption, Instrument, InstrumentAny, OptionContract,
            stubs::{audusd_sim, crypto_option_btc_deribit, option_contract_appl},
        },
        orders::{
            Order, OrderAny,
            builder::OrderTestBuilder,
            stubs::{TestOrderEventStubs, TestOrderStubs},
        },
        types::{Currency, Money, Price, Quantity},
    };
    use rstest::rstest;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use super::{
        CappedOptionFeeModel, FeeModel, FeeModelAny, FixedFeeModel, MakerTakerFeeModel,
        PerContractFeeModel, PolymarketFeeModel, TieredNotionalOptionFeeModel,
    };

    #[rstest]
    fn test_fixed_model_single_fill() {
        let expected_commission = Money::new(1.0, Currency::USD());
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let fee_model = FixedFeeModel::new(expected_commission, None).unwrap();
        let market_order = OrderTestBuilder::new(OrderType::Market)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Buy)
            .quantity(Quantity::from(100_000))
            .build();
        let accepted_order = TestOrderStubs::make_accepted_order(&market_order);
        let commission = fee_model
            .get_commission(
                &accepted_order,
                Quantity::from(100_000),
                Price::from("1.0"),
                &aud_usd,
            )
            .unwrap();
        assert_eq!(commission, expected_commission);
    }

    #[rstest]
    #[case(OrderSide::Buy, true, Money::from("1 USD"), Money::from("0 USD"))]
    #[case(OrderSide::Sell, true, Money::from("1 USD"), Money::from("0 USD"))]
    #[case(OrderSide::Buy, false, Money::from("1 USD"), Money::from("1 USD"))]
    #[case(OrderSide::Sell, false, Money::from("1 USD"), Money::from("1 USD"))]
    fn test_fixed_model_multiple_fills(
        #[case] order_side: OrderSide,
        #[case] charge_commission_once: bool,
        #[case] expected_first_fill: Money,
        #[case] expected_next_fill: Money,
    ) {
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let fee_model =
            FixedFeeModel::new(expected_first_fill, Some(charge_commission_once)).unwrap();
        let market_order = OrderTestBuilder::new(OrderType::Market)
            .instrument_id(aud_usd.id())
            .side(order_side)
            .quantity(Quantity::from(100_000))
            .build();
        let mut accepted_order = TestOrderStubs::make_accepted_order(&market_order);
        let commission_first_fill = fee_model
            .get_commission(
                &accepted_order,
                Quantity::from(50_000),
                Price::from("1.0"),
                &aud_usd,
            )
            .unwrap();
        let fill = TestOrderEventStubs::filled(
            &accepted_order,
            &aud_usd,
            None,
            None,
            None,
            Some(Quantity::from(50_000)),
            None,
            None,
            None,
            None,
        );
        accepted_order.apply(fill).unwrap();
        let commission_next_fill = fee_model
            .get_commission(
                &accepted_order,
                Quantity::from(50_000),
                Price::from("1.0"),
                &aud_usd,
            )
            .unwrap();
        assert_eq!(commission_first_fill, expected_first_fill);
        assert_eq!(commission_next_fill, expected_next_fill);
    }

    #[rstest]
    fn test_maker_taker_fee_model_maker_commission() {
        let fee_model = MakerTakerFeeModel;
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let maker_fee = aud_usd.maker_fee();
        let price = Price::from("1.0");
        let limit_order = OrderTestBuilder::new(OrderType::Limit)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Sell)
            .price(price)
            .quantity(Quantity::from(100_000))
            .build();
        let fill = TestOrderStubs::make_filled_order(&limit_order, &aud_usd, LiquiditySide::Maker);
        let expected_commission = fill.quantity().as_decimal() * price.as_decimal() * maker_fee;
        let commission = fee_model
            .get_commission(&fill, Quantity::from(100_000), Price::from("1.0"), &aud_usd)
            .unwrap();
        assert_eq!(commission.as_decimal(), expected_commission);
    }

    #[rstest]
    fn test_maker_taker_fee_model_uses_decimal_rounding() {
        let fee_model = MakerTakerFeeModel;
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let price = Price::from("1.0");
        let quantity = Quantity::from("117250");
        let limit_order = OrderTestBuilder::new(OrderType::Limit)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Sell)
            .price(price)
            .quantity(quantity)
            .build();
        let fill = TestOrderStubs::make_filled_order(&limit_order, &aud_usd, LiquiditySide::Maker);

        let commission = fee_model
            .get_commission(&fill, quantity, price, &aud_usd)
            .unwrap();

        assert_eq!(commission, Money::from("2.34 USD"));
    }

    #[rstest]
    fn test_maker_taker_fee_model_taker_commission() {
        let fee_model = MakerTakerFeeModel;
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let taker_fee = aud_usd.taker_fee();
        let price = Price::from("1.0");
        let limit_order = OrderTestBuilder::new(OrderType::Limit)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Sell)
            .price(price)
            .quantity(Quantity::from(100_000))
            .build();

        let fill = TestOrderStubs::make_filled_order(&limit_order, &aud_usd, LiquiditySide::Taker);
        let expected_commission = fill.quantity().as_decimal() * price.as_decimal() * taker_fee;
        let commission = fee_model
            .get_commission(&fill, Quantity::from(100_000), Price::from("1.0"), &aud_usd)
            .unwrap();
        assert_eq!(commission.as_decimal(), expected_commission);
    }

    #[rstest]
    fn test_per_contract_fee_model() {
        let commission_per_contract = Money::new(0.50, Currency::USD());
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let fee_model = PerContractFeeModel::new(commission_per_contract).unwrap();
        let market_order = OrderTestBuilder::new(OrderType::Market)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Buy)
            .quantity(Quantity::from(100))
            .build();
        let accepted_order = TestOrderStubs::make_accepted_order(&market_order);
        let commission = fee_model
            .get_commission(
                &accepted_order,
                Quantity::from(100),
                Price::from("1.0"),
                &aud_usd,
            )
            .unwrap();
        assert_eq!(commission, Money::new(50.0, Currency::USD()));
    }

    #[rstest]
    fn test_per_contract_fee_model_partial_fill() {
        let commission_per_contract = Money::new(1.25, Currency::USD());
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let fee_model = PerContractFeeModel::new(commission_per_contract).unwrap();
        let market_order = OrderTestBuilder::new(OrderType::Market)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Sell)
            .quantity(Quantity::from(1000))
            .build();
        let accepted_order = TestOrderStubs::make_accepted_order(&market_order);
        let commission = fee_model
            .get_commission(
                &accepted_order,
                Quantity::from(400),
                Price::from("1.0"),
                &aud_usd,
            )
            .unwrap();
        assert_eq!(commission, Money::new(500.0, Currency::USD()));
    }

    #[rstest]
    fn test_per_contract_fee_model_uses_decimal_rounding() {
        let commission_per_contract = Money::from("0.50 USD");
        let aud_usd = InstrumentAny::CurrencyPair(audusd_sim());
        let fee_model = PerContractFeeModel::new(commission_per_contract).unwrap();
        let market_order = OrderTestBuilder::new(OrderType::Market)
            .instrument_id(aud_usd.id())
            .side(OrderSide::Buy)
            .quantity(Quantity::from("5"))
            .build();
        let accepted_order = TestOrderStubs::make_accepted_order(&market_order);

        let commission = fee_model
            .get_commission(
                &accepted_order,
                Quantity::from("4.69"),
                Price::from("1.0"),
                &aud_usd,
            )
            .unwrap();

        assert_eq!(commission, Money::from("2.34 USD"));
    }

    #[rstest]
    fn test_per_contract_fee_model_negative_commission_fails() {
        let result = PerContractFeeModel::new(Money::new(-1.0, Currency::USD()));
        assert!(result.is_err());
    }

    #[rstest]
    #[case::maker(Some(dec!(-0.0001)), Some(dec!(0.0003)), None, "maker_rate")]
    #[case::taker(Some(dec!(0.0001)), Some(dec!(-0.0003)), None, "taker_rate")]
    #[case::cap(Some(dec!(0.0001)), Some(dec!(0.0003)), Some(dec!(-0.125)), "cap_rate")]
    fn test_capped_option_fee_model_negative_rate_fails(
        #[case] maker_rate: Option<Decimal>,
        #[case] taker_rate: Option<Decimal>,
        #[case] cap_rate: Option<Decimal>,
        #[case] expected_field: &str,
    ) {
        let result = CappedOptionFeeModel::new(maker_rate, taker_rate, cap_rate);

        assert_eq!(
            result.unwrap_err().to_string(),
            format!("`{expected_field}` must be greater than or equal to zero")
        );
    }

    #[rstest]
    fn test_capped_option_fee_model_maker_commission_rate_bound(
        crypto_option_btc_deribit: CryptoOption,
    ) {
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let fill = option_fill_order(&instrument, LiquiditySide::Maker);
        let fee_model = FeeModelAny::CappedOption(
            CappedOptionFeeModel::new(Some(dec!(0.0001)), Some(dec!(0.0003)), None).unwrap(),
        );

        let commission = fee_model
            .get_commission_with_context(
                &fill,
                Quantity::from("2.0"),
                Price::from("100.00"),
                &instrument,
                Some(Price::from("50000.00")),
            )
            .unwrap();

        assert_eq!(commission.currency, Currency::USD());
        assert_eq!(commission.as_decimal(), dec!(10.00));
    }

    #[rstest]
    fn test_capped_option_fee_model_taker_commission_cap_bound(
        crypto_option_btc_deribit: CryptoOption,
    ) {
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let fill = option_fill_order(&instrument, LiquiditySide::Taker);
        let fee_model =
            CappedOptionFeeModel::new(Some(dec!(0.0001)), Some(dec!(0.0003)), None).unwrap();

        let commission = fee_model
            .get_commission_with_context(
                &fill,
                Quantity::from("2.0"),
                Price::from("10.00"),
                &instrument,
                Some(Price::from("50000.00")),
            )
            .unwrap();

        assert_eq!(commission.currency, Currency::USD());
        assert_eq!(commission.as_decimal(), dec!(2.50));
    }

    #[rstest]
    fn test_capped_option_fee_model_applies_contract_multiplier(
        mut option_contract_appl: OptionContract,
    ) {
        option_contract_appl.multiplier = Quantity::from(100);
        let instrument = InstrumentAny::OptionContract(option_contract_appl);
        let fill = option_fill_order(&instrument, LiquiditySide::Maker);
        let fee_model =
            CappedOptionFeeModel::new(Some(dec!(0.0001)), Some(dec!(0.0003)), None).unwrap();

        let commission = fee_model
            .get_commission_with_context(
                &fill,
                Quantity::from("2"),
                Price::from("2.00"),
                &instrument,
                Some(Price::from("150.00")),
            )
            .unwrap();

        assert_eq!(commission.currency, Currency::USD());
        assert_eq!(commission.as_decimal(), dec!(3.00));
    }

    #[rstest]
    fn test_capped_option_fee_model_inverse_commission_uses_settlement_currency(
        mut crypto_option_btc_deribit: CryptoOption,
    ) {
        crypto_option_btc_deribit.is_inverse = true;
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let fill = option_fill_order(&instrument, LiquiditySide::Taker);
        let fee_model =
            CappedOptionFeeModel::new(Some(dec!(0.0001)), Some(dec!(0.0003)), None).unwrap();

        let commission = fee_model
            .get_commission(
                &fill,
                Quantity::from("2.0"),
                Price::from("0.010"),
                &instrument,
            )
            .unwrap();

        assert_eq!(commission.currency, Currency::BTC());
        assert_eq!(commission.as_decimal(), dec!(0.0006));
    }

    #[rstest]
    fn test_capped_option_fee_model_requires_underlying_price(
        crypto_option_btc_deribit: CryptoOption,
    ) {
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let fill = option_fill_order(&instrument, LiquiditySide::Taker);
        let fee_model = CappedOptionFeeModel::default();

        let result = fee_model.get_commission(
            &fill,
            Quantity::from("1.0"),
            Price::from("10.00"),
            &instrument,
        );

        assert!(result.is_err());
    }

    #[rstest]
    fn test_capped_option_fee_model_rejects_non_option_instrument() {
        let instrument = InstrumentAny::CurrencyPair(audusd_sim());
        let fill = option_fill_order(&instrument, LiquiditySide::Taker);
        let fee_model = CappedOptionFeeModel::default();

        let result = fee_model.get_commission_with_context(
            &fill,
            Quantity::from("1.0"),
            Price::from("10.00"),
            &instrument,
            Some(Price::from("50000.00")),
        );

        assert!(result.is_err());
    }

    #[rstest]
    #[case::maker(LiquiditySide::Maker, dec!(0.04))]
    #[case::taker(LiquiditySide::Taker, dec!(0.10))]
    fn test_tiered_notional_option_fee_model_commission(
        crypto_option_btc_deribit: CryptoOption,
        #[case] liquidity_side: LiquiditySide,
        #[case] expected_commission: Decimal,
    ) {
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let fill = option_fill_order(&instrument, liquidity_side);
        let fee_model = FeeModelAny::TieredNotionalOption(
            TieredNotionalOptionFeeModel::new(Some(dec!(0.0002)), Some(dec!(0.0005))).unwrap(),
        );

        let commission = fee_model
            .get_commission(
                &fill,
                Quantity::from("2.0"),
                Price::from("100.00"),
                &instrument,
            )
            .unwrap();

        assert_eq!(commission.currency, Currency::USD());
        assert_eq!(commission.as_decimal(), expected_commission);
    }

    #[rstest]
    fn test_tiered_notional_option_fee_model_inverse_commission_uses_base_currency(
        mut crypto_option_btc_deribit: CryptoOption,
    ) {
        crypto_option_btc_deribit.is_inverse = true;
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let fill = option_fill_order(&instrument, LiquiditySide::Taker);
        let fee_model =
            TieredNotionalOptionFeeModel::new(Some(dec!(0.0002)), Some(dec!(0.0005))).unwrap();

        let commission = fee_model
            .get_commission(
                &fill,
                Quantity::from("2.0"),
                Price::from("0.010"),
                &instrument,
            )
            .unwrap();

        assert_eq!(commission.currency, Currency::BTC());
        assert_eq!(commission.as_decimal(), dec!(0.10));
    }

    #[rstest]
    fn test_tiered_notional_option_fee_model_rejects_non_option_instrument() {
        let instrument = InstrumentAny::CurrencyPair(audusd_sim());
        let fill = option_fill_order(&instrument, LiquiditySide::Taker);
        let fee_model = TieredNotionalOptionFeeModel::default();

        let result = fee_model.get_commission(
            &fill,
            Quantity::from("1.0"),
            Price::from("10.00"),
            &instrument,
        );

        assert!(result.is_err());
    }

    #[rstest]
    #[case::maker(Some(dec!(-0.0002)), Some(dec!(0.0005)), "maker_rate")]
    #[case::taker(Some(dec!(0.0002)), Some(dec!(-0.0005)), "taker_rate")]
    fn test_tiered_notional_option_fee_model_negative_rate_fails(
        #[case] maker_rate: Option<Decimal>,
        #[case] taker_rate: Option<Decimal>,
        #[case] expected_field: &str,
    ) {
        let result = TieredNotionalOptionFeeModel::new(maker_rate, taker_rate);

        assert_eq!(
            result.unwrap_err().to_string(),
            format!("`{expected_field}` must be greater than or equal to zero")
        );
    }

    #[rstest]
    fn test_tiered_notional_option_fee_model_requires_liquidity_side(
        crypto_option_btc_deribit: CryptoOption,
    ) {
        let instrument = InstrumentAny::CryptoOption(crypto_option_btc_deribit);
        let order = OrderTestBuilder::new(OrderType::Limit)
            .instrument_id(instrument.id())
            .side(OrderSide::Buy)
            .price(Price::from("100.00"))
            .quantity(Quantity::from("2.0"))
            .build();
        let fee_model = TieredNotionalOptionFeeModel::default();

        let result = fee_model.get_commission(
            &order,
            Quantity::from("1.0"),
            Price::from("10.00"),
            &instrument,
        );

        assert!(result.is_err());
    }

    fn option_fill_order(instrument: &InstrumentAny, liquidity_side: LiquiditySide) -> OrderAny {
        let limit_order = OrderTestBuilder::new(OrderType::Limit)
            .instrument_id(instrument.id())
            .side(OrderSide::Buy)
            .price(Price::from("100.00"))
            .quantity(Quantity::from("2.0"))
            .build();

        TestOrderStubs::make_filled_order(&limit_order, instrument, liquidity_side)
    }

    // Build a Polymarket BinaryOption stub with the supplied taker fee.
    fn polymarket_binary_option(taker_fee: Decimal) -> InstrumentAny {
        use chrono::{TimeZone, Utc};
        use nautilus_core::UnixNanos;
        use nautilus_model::{
            enums::AssetClass,
            identifiers::{InstrumentId, Symbol},
            instruments::BinaryOption,
        };

        let raw_symbol = Symbol::new("polymarket-test-market");
        let activation = Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap();
        let expiration = Utc.with_ymd_and_hms(2026, 5, 1, 0, 5, 0).unwrap();
        let price_increment = Price::from("0.01");
        let size_increment = Quantity::from("0.01");
        let inst = BinaryOption::new(
            InstrumentId::from("test-market.POLYMARKET"),
            raw_symbol,
            AssetClass::Alternative,
            Currency::USDC(),
            UnixNanos::from(activation.timestamp_nanos_opt().unwrap() as u64),
            UnixNanos::from(expiration.timestamp_nanos_opt().unwrap() as u64),
            price_increment.precision,
            size_increment.precision,
            price_increment,
            size_increment,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(taker_fee),
            None,
            UnixNanos::default(),
            UnixNanos::default(),
        );
        InstrumentAny::BinaryOption(inst)
    }

    fn polymarket_fill(instrument: &InstrumentAny, liquidity_side: LiquiditySide) -> OrderAny {
        let limit_order = OrderTestBuilder::new(OrderType::Limit)
            .instrument_id(instrument.id())
            .side(OrderSide::Buy)
            .price(Price::from("0.50"))
            .quantity(Quantity::from("100"))
            .build();
        TestOrderStubs::make_filled_order(&limit_order, instrument, liquidity_side)
    }

    #[rstest]
    #[case::p10(Price::from("0.10"), dec!(0.63))]
    #[case::p30(Price::from("0.30"), dec!(1.47))]
    #[case::p50(Price::from("0.50"), dec!(1.75))]
    #[case::p70(Price::from("0.70"), dec!(1.47))]
    #[case::p90(Price::from("0.90"), dec!(0.63))]
    fn test_polymarket_fee_model_taker_curve(
        #[case] fill_px: Price,
        #[case] expected: Decimal,
    ) {
        // qty=100, rate=0.07. Formula: 100 * 0.07 * p * (1-p).
        let instrument = polymarket_binary_option(dec!(0.07));
        let fill = polymarket_fill(&instrument, LiquiditySide::Taker);
        let fee_model = PolymarketFeeModel::new(None);

        let commission = fee_model
            .get_commission(&fill, Quantity::from("100"), fill_px, &instrument)
            .unwrap();

        assert_eq!(commission.currency, Currency::USDC());
        assert_eq!(commission.as_decimal(), expected);
    }

    #[rstest]
    fn test_polymarket_fee_model_maker_disabled() {
        let instrument = polymarket_binary_option(dec!(0.07));
        let fill = polymarket_fill(&instrument, LiquiditySide::Maker);
        let fee_model = PolymarketFeeModel::new(Some(false));

        let commission = fee_model
            .get_commission(&fill, Quantity::from("100"), Price::from("0.50"), &instrument)
            .unwrap();

        assert_eq!(commission, Money::zero(Currency::USDC()));
    }

    #[rstest]
    fn test_polymarket_fee_model_maker_rebate_crypto() {
        // qty=100, rate=0.07, p=0.5, crypto rebate share 0.20.
        // fee_equivalent = 100 * 0.07 * 0.5 * 0.5 = 1.75
        // rebate = 1.75 * 0.20 = 0.35 -> credited as -0.35
        let instrument = polymarket_binary_option(dec!(0.07));
        let fill = polymarket_fill(&instrument, LiquiditySide::Maker);
        let fee_model = PolymarketFeeModel::new(Some(true));

        let commission = fee_model
            .get_commission(&fill, Quantity::from("100"), Price::from("0.50"), &instrument)
            .unwrap();

        assert_eq!(commission.as_decimal(), dec!(-0.35));
    }

    #[rstest]
    fn test_polymarket_fee_model_maker_rebate_non_crypto() {
        // qty=100, rate=0.04 (politics), p=0.5, non-crypto rebate share 0.25.
        // fee_equivalent = 100 * 0.04 * 0.5 * 0.5 = 1.00
        // rebate = 1.00 * 0.25 = 0.25 -> credited as -0.25
        let instrument = polymarket_binary_option(dec!(0.04));
        let fill = polymarket_fill(&instrument, LiquiditySide::Maker);
        let fee_model = PolymarketFeeModel::new(Some(true));

        let commission = fee_model
            .get_commission(&fill, Quantity::from("100"), Price::from("0.50"), &instrument)
            .unwrap();

        assert_eq!(commission.as_decimal(), dec!(-0.25));
    }

    #[rstest]
    fn test_polymarket_fee_model_zero_rate_returns_zero() {
        // Geopolitics markets have taker_fee=0.
        let instrument = polymarket_binary_option(Decimal::ZERO);
        let fill = polymarket_fill(&instrument, LiquiditySide::Taker);
        let fee_model = PolymarketFeeModel::default();

        let commission = fee_model
            .get_commission(&fill, Quantity::from("100"), Price::from("0.50"), &instrument)
            .unwrap();

        assert_eq!(commission, Money::zero(Currency::USDC()));
    }
}
