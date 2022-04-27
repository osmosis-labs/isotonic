use cosmwasm_std::Decimal;
use derivative::Derivative;
use isotonic_credit_agency::msg::MarketConfig;
use isotonic_market::state::SECONDS_IN_YEAR;
use utils::{interest::Interest, token::Token};

#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct MarketBuilder {
    market_token: String,
    #[derivative(Default(value = "Decimal::percent(50)"))]
    collateral_ratio: Decimal,
    #[derivative(Default(
        value = "Interest::Linear {base: Decimal::percent(3), slope: Decimal::percent(20)}"
    ))]
    interest_rate: Interest,
    #[derivative(Default(value = "SECONDS_IN_YEAR as u64"))]
    charge_period: u64,
    #[derivative(Default(value = "Decimal::zero()"))]
    reserve_factor: Decimal,
}

impl MarketBuilder {
    pub fn new(market_token: &str) -> Self {
        Self {
            market_token: market_token.to_string(),
            ..Default::default()
        }
    }

    pub fn with_collateral_ratio(mut self, collateral_ratio: Decimal) -> Self {
        self.collateral_ratio = collateral_ratio;
        self
    }

    pub fn with_linear_interest(mut self, base: Decimal, slope: Decimal) -> Self {
        self.interest_rate = Interest::Linear { base, slope };
        self
    }

    pub fn with_piecewise_interest(
        mut self,
        base: Decimal,
        slope1: Decimal,
        slope2: Decimal,
        optimal_utilisation: Decimal,
    ) -> Self {
        self.interest_rate = Interest::PiecewiseLinear {
            base,
            slope1,
            slope2,
            optimal_utilisation,
        };
        self
    }

    pub fn with_charge_period(mut self, charge_period: u64) -> Self {
        self.charge_period = charge_period;
        self
    }

    pub fn with_reserve_factor(mut self, reserve_factor: Decimal) -> Self {
        self.reserve_factor = reserve_factor;
        self
    }

    pub(crate) fn build(self, price_oracle: &str) -> MarketConfig {
        MarketConfig {
            name: self.market_token.clone(),
            symbol: self.market_token.clone(),
            decimals: 9,
            market_token: Token::Native(self.market_token),
            market_cap: None,
            interest_rate: self.interest_rate,
            interest_charge_period: self.charge_period,
            collateral_ratio: self.collateral_ratio,
            price_oracle: price_oracle.to_string(),
            reserve_factor: self.reserve_factor,
        }
    }
}
