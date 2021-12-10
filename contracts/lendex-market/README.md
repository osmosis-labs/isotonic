# Market contract

The Market contract is the entry point for all lending and borrowing for one base asset.

## Instantiate

Prototype (not yet implemented) of instantiate message.
```rust
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub token_id: u64,
    pub base_asset: String,
}
```

## Messages

TBD

## Queries

`Configuration {}` - returns current configuration, for example addresses of both btoken and ltokens

`TransferableAmount { token, account }` - queries token of given `token` address for amount available to transfer from account of address `account`. Note: `btoken`'s address will always return 0
