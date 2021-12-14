use super::suite::SuiteBuilder;

use crate::msg::InterestResponse;
use cosmwasm_std::{coin, Coin, Decimal};

#[test]
fn query_interest() {
    let lender = "lender";
    let borrower = "borrower";
    let base_asset = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(150, base_asset)])
        .with_base_asset(base_asset)
        .build();

    // At first, the lender has no l-token, and the contract has no base asset.
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 0);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);

    // And, we are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero()
        },
        resp
    );

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(100, base_asset)])
        .unwrap();

    // After the deposit, the lender has 100 l-token and the contract has 100 base asset.
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 100);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);

    // We still are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero()
        },
        resp
    );

    // Borrow some tokens
    suite.borrow(borrower, 10).unwrap();

    // Now utilisation is 10% (10/100),
    // The interest changed according to the linear formula: 3% + 20% * 10% = 3% + 2% = 5%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(10),
            interest: Decimal::percent(3) + Decimal::percent(2),
        },
        resp
    );

    // Repay some tokens
    suite.repay(borrower, Coin::new(5, base_asset)).unwrap();

    // Utilisation is now 5% ((10-5)/100).
    // The interest changed according to the linear formula: 3% + 20% * 5% = 3% + 1% = 4%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(5),
            interest: Decimal::percent(3) + Decimal::percent(1),
        },
        resp
    );

    // Lend some more
    suite.deposit(lender, &[Coin::new(50, base_asset)]).unwrap();

    // Utilisation is now ~3.33% ((10-5)/(100+50)).
    // The interest changed according to the linear formula: 3% + 20% * 3.33% = 3% + 0.67% = 3.67%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::from_ratio(1u8, 30u8),
            interest: Decimal::percent(3) + Decimal::from_ratio(1u8, 150u8),
        },
        resp
    );
}
