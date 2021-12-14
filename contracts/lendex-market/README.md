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

`Deposit {}` - deposits funds. If denom matches `base_asset`, proper amount of tokens will be minted and sent to ltoken account

`Withdraw { amount: Uint128 }` - requests to withdraw the amount of ltokens

`Borrow { amount: Uint128 }` - if contract has some balance, borrow request can be sent. It will mint proper amount of btokens into btoken contracts and separately to borrower address

`Repay {}` - repays borrowed tokens. Any surplus is sent back to borrower

## Queries

`Configuration {}` - returns current configuration, for example addresses of both btoken and ltokens

`TransferableAmount { token, account }` - queries token of given `token` address for amount available to transfer from account of address `account`. Note: `btoken`'s address will always return 0
