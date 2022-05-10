use cosmwasm_std::coin;

use super::suite::SuiteBuilder;

#[test]
fn distribute_as_ltokens_works() {
    let alice = "alice";
    let bob = "bob";
    let mut suite = SuiteBuilder::new()
        .with_funds(alice, &[coin(100, "ATOM")])
        .with_funds(bob, &[coin(300, "ATOM")])
        .with_ca_funds(&[coin(20, "ATOM")])
        .with_market_token("ATOM")
        .build();

    suite.deposit(alice, &[coin(100, "ATOM")]).unwrap();
    suite.deposit(bob, &[coin(300, "ATOM")]).unwrap();

    assert_eq!(suite.query_contract_asset_balance().unwrap(), 400);
    assert_eq!(suite.query_ltoken_balance(alice).unwrap().u128(), 100);
    assert_eq!(suite.query_ltoken_balance(bob).unwrap().u128(), 300);

    suite
        .distribute_as_ltokens(&suite.credit_agency(), coin(20, "ATOM"))
        .unwrap();

    assert_eq!(suite.query_contract_asset_balance().unwrap(), 420);
    assert_eq!(suite.query_ltoken_balance(alice).unwrap().u128(), 105);
    assert_eq!(suite.query_ltoken_balance(bob).unwrap().u128(), 315);
}
