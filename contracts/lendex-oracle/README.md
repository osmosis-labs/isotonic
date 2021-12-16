# Price Oracle contract

The Price Oracle provides info about the sell/buy prices of a specific asset in terms
of the common token used by the credit agency.

## Instantiate

```rust
pub struct InstantiateMsg {
    pub oracle: String,        // the address of the oracle
    pub maximum_age: Duration, // maximum time after which a record is considered to be outdated
}
```

## Messages

`SetPrice { sell: String, buy: String, rate: Decimal }` - sets the exchange rate for a trading pair.
Only the oracle set on instantiation can call this. Any other caller will get an `Unauthorized`
error.

## Queries

`Configuration {}` - returns current configuration

```rust
pub struct Config {
    pub oracle: Addr,
    pub maximum_age: Duration,
}
```

`Price { sell: String, buy: String }` - returns the exchange rate between two denoms as a
`PriceResponse`.

```rust
pub struct PriceResponse {
    pub rate: Decimal,
}
```
