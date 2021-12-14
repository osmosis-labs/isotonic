use super::suite::SuiteBuilder;

use cosmwasm_std::coin;

#[test]
fn borrow_works() {
    let borrower = "borrower";
    let mut suite = SuiteBuilder::new()
        .with_contract_funds(coin(150, "ATOM"))
        .with_base_asset("ATOM")
        .build();

    // At first, the lender has no l-token, and the contract has some base assets
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 0);

    // Borrow some tokens
    suite.borrow(borrower, 100).unwrap();

    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);
    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 100);
}

#[test]
fn borrow_and_repay() {
    let borrower = "borrower";
    let base_asset = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_contract_funds(coin(150, base_asset))
        .with_base_asset(base_asset)
        .build();

    // Borrow some tokens
    suite.borrow(borrower, 100).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 50);

    // Repay all borrowed tokens
    suite.repay(borrower, coin(100, base_asset)).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 150);
    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 0);
}

#[test]
fn repay_small_amounts() {
    let borrower = "borrower";
    let base_asset = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_contract_funds(coin(100, base_asset))
        .with_base_asset(base_asset)
        .build();

    // Borrow some tokens
    suite.borrow(borrower, 100).unwrap();

    // Repay some borrowed tokens
    suite.repay(borrower, coin(33, base_asset)).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 33);
    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 67);

    suite.repay(borrower, coin(67, base_asset)).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 0);
}

#[test]
fn overpay_repay() {
    let borrower = "borrower";
    let base_asset = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_funds(borrower, &[coin(50, base_asset)])
        .with_contract_funds(coin(100, base_asset))
        .with_base_asset(base_asset)
        .build();

    // Borrow some tokens
    suite.borrow(borrower, 100).unwrap();

    // Overpay borrowed tokens - 120 instead of 100
    suite.repay(borrower, coin(120, base_asset)).unwrap();
    // Contract will still have only initial 100 tokens, since it sends
    // surplus back to borrower
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    // No more btokens
    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 0);
    // Initial amount - surplus was returned
    assert_eq!(suite.query_asset_balance(borrower).unwrap(), 50);
}
