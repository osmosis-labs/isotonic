use cosmwasm_std::{Decimal, Deps, Env, Uint128};
use isotonic_token::msg::TokenInfoResponse;

use crate::{
    state::{Config, TokensInfo, CONFIG, RESERVE, SECONDS_IN_YEAR},
    ContractError,
};

/// Values that should be updated when interest is charged for all pending charge periods
pub struct InterestUpdate {
    /// The new RESERVE value
    pub reserve: Uint128,
    /// The ratio to rebase LTokens by
    pub ltoken_ratio: Decimal,
    /// The ratio to rebase BTokens by
    pub btoken_ratio: Decimal,
}

pub fn epochs_passed(cfg: &Config, env: Env) -> Result<u64, ContractError> {
    Ok((env.block.time.seconds() - cfg.last_charged) / cfg.interest_charge_period)
}

/// Calculates new values after applying all pending interest charges
pub fn calculate_interest(
    deps: Deps,
    epochs_passed: u64,
) -> Result<Option<InterestUpdate>, ContractError> {
    if epochs_passed == 0 {
        return Ok(None);
    }

    let cfg = CONFIG.load(deps.storage)?;

    let charged_time = epochs_passed * cfg.interest_charge_period;

    let tokens_info = token_supply(deps, &cfg)?;

    let supplied = tokens_info.ltoken.total_supply.display_amount();
    let borrowed = tokens_info.btoken.total_supply.display_amount();

    // safety - if there are no ltokens, don't charge interest (would panic later)
    if supplied == Uint128::zero() {
        return Ok(None);
    }

    let interest = cfg.rates.calculate_interest_rate(utilisation(&tokens_info));

    // bMul = calculate_interest() * epochs_passed * epoch_length / SECONDS_IN_YEAR
    let btoken_ratio: Decimal =
        interest * Decimal::from_ratio(charged_time as u128, SECONDS_IN_YEAR);

    let old_reserve = RESERVE.load(deps.storage)?;
    // Add to reserve only portion of money charged here
    let charged_interest = btoken_ratio * borrowed;
    let reserve = old_reserve + cfg.reserve_factor * charged_interest;

    // remember to add old reserve balance into supplied tokens
    let base_asset_balance = supplied + old_reserve - borrowed;

    let l_supply = borrowed + base_asset_balance - reserve;

    // lMul = b_supply() * ratio / l_supply
    let ltoken_ratio: Decimal = Decimal::from_ratio(borrowed * btoken_ratio, l_supply);

    Ok(Some(InterestUpdate {
        reserve,
        ltoken_ratio,
        btoken_ratio,
    }))
}

/// Figure out the current utilisation
pub fn utilisation(tokens_info: &TokensInfo) -> Decimal {
    if tokens_info.ltoken.total_supply.is_zero() {
        Decimal::zero()
    } else {
        Decimal::from_ratio(
            tokens_info.btoken.total_supply.display_amount(),
            tokens_info.ltoken.total_supply.display_amount(),
        )
    }
}

pub fn token_supply(deps: Deps, config: &Config) -> Result<TokensInfo, ContractError> {
    let ltoken_contract = &config.ltoken_contract;
    let ltoken: TokenInfoResponse = deps.querier.query_wasm_smart(
        ltoken_contract,
        &isotonic_token::msg::QueryMsg::TokenInfo {},
    )?;
    let btoken_contract = &config.btoken_contract;
    let btoken: TokenInfoResponse = deps.querier.query_wasm_smart(
        btoken_contract,
        &isotonic_token::msg::QueryMsg::TokenInfo {},
    )?;
    Ok(TokensInfo { ltoken, btoken })
}
