# Credit Agency contract

The Credit Agency is the entity that sets up market contracts for different assets and
aggregates information about an account's debt/collateral for use in borrow/transfer checks.

## Instantiate

``` rust
pub struct InstantiateMsg {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: String,
    /// The CodeId of the lendex-market contract
    pub lendex_market_id: u64,
    /// The CodeId of the lendex-token contract
    pub lendex_token_id: u64,
    /// Token denom which would be distributed as reward token to lendex token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: String,
}
```

## Messages

`CreateMarket(MarketConfig)` - creates a new market using the provided market configuration.
Only the gov contract set in instantiation may call this. This must be the first and only
market set up for the given base asset - if there was a previous call of `CreateMarket`,
one of these errors might be returned:

``` rust
    #[error("A market for base asset {0} is still being created")]
    MarketCreating(String),

    #[error("A market for base asset {0} already exists")]
    MarketAlreadyExists(String),
```

A `MarketConfig` looks like this:

``` rust
pub struct MarketConfig {
    /// Name used to create the sub-tokens `Lent ${name}` and `Borrowed ${name}`
    pub name: String,
    /// Symbol used to create the sub-tokens `L${symbol}` and `B${symbol}`
    pub symbol: String,
    /// Decimals for sub-tokens `L` and `B`
    pub decimals: u8,
    /// Native denom for the base asset
    pub base_asset: String,
    /// Interest rate curve
    pub interest_rate: Interest,
}
```


## Queries

`Configuration {}` returns the current configuration for this contract.

``` rust
pub struct Config {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: Addr,
    /// The CodeId of the lendex-market contract
    pub lendex_market_id: u64,
    /// The CodeId of the lendex-token contract
    pub lendex_token_id: u64,
    /// Token denom which would be distributed as reward token to lendex token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: String,
}
```

`Market { base_asset: String }` returns a `MarketResponse` with the address of a market, or an
error in case the market doesn't exist or is still being instantiated.

``` rust
pub struct MarketResponse {
    pub base_asset: String,
    pub market: Addr,
}
```

`ListMarkets {start_after: Option<String>, limit: Option<u32>}` returns a `ListMarketResponse`.
If pagination values aren't configured, the defaults start from the first market on the list
and limit them to 10 per page. Maximum settable limit is 30.

``` rust
pub struct ListMarketsResponse {
    pub markets: Vec<MarketResponse>,
}
```
