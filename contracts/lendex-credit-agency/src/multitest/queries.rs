use super::suite::SuiteBuilder;

#[test]
fn query_market() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite.create_market_quick("gov", "osmo", "OSMO").unwrap();
    let res = suite.query_market("OSMO").unwrap();
    assert_eq!(res.base_asset, "OSMO");
}

#[test]
fn query_market_does_not_exist() {
    let suite = SuiteBuilder::new().with_gov("gov").build();

    let err = suite.query_market("OSMO").unwrap_err();
    assert!(err
        .to_string()
        .ends_with("No market set up for base asset OSMO"));
}

// TODO: a test for when a market is not yet instantiated? or would that be too hard to mock?

#[test]
fn list_markets() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite.create_market_quick("gov", "osmo", "OSMO").unwrap();
    suite.create_market_quick("gov", "atom", "ATOM").unwrap();
    suite.create_market_quick("gov", "btc", "BTC").unwrap();
    let mut list: Vec<_> = suite
        .list_markets()
        .unwrap()
        .markets
        .into_iter()
        .map(|r| r.base_asset)
        .collect();
    list.sort();

    assert_eq!(list, ["ATOM", "BTC", "OSMO"]);
}

#[test]
fn list_markets_empty_list() {
    let suite = SuiteBuilder::new().with_gov("gov").build();

    let res = suite.list_markets().unwrap();
    assert_eq!(res.markets, []);
}

fn generate_denoms(prefix: &str, start: u32, end: u32) -> Vec<String> {
    (start..end)
        .into_iter()
        .map(|i| format!("{}{:02}", prefix, i))
        .collect()
}

#[test]
fn list_markets_default_pagination() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    // create markets for native tokens "TOKEN00", "TOKEN01", ..., "TOKEN14"
    // the default pagination limit is 10 entries per page
    for i in 0..15 {
        suite
            .create_market_quick("gov", &format!("token{:02}", i), &format!("TOKEN{:02}", i))
            .unwrap();
    }

    let mut list1: Vec<_> = suite
        .list_markets()
        .unwrap()
        .markets
        .into_iter()
        .map(|r| r.base_asset)
        .collect();
    list1.sort();
    assert_eq!(list1, generate_denoms("TOKEN", 0, 10));

    let mut list2: Vec<_> = suite
        .list_markets_with_pagination(list1.last().unwrap().clone(), None)
        .unwrap()
        .markets
        .into_iter()
        .map(|r| r.base_asset)
        .collect();
    list2.sort();
    assert_eq!(list2, generate_denoms("TOKEN", 10, 15));
}

#[test]
fn list_markets_custom_pagination() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    // create markets for native tokens "TOKEN00", "TOKEN01", ..., "TOKEN05"
    // we set the pagination limit to 3 entries per page
    for i in 0..5 {
        suite
            .create_market_quick("gov", &format!("token{:02}", i), &format!("TOKEN{:02}", i))
            .unwrap();
    }

    let mut list1: Vec<_> = suite
        .list_markets_with_pagination(None, 3)
        .unwrap()
        .markets
        .into_iter()
        .map(|r| r.base_asset)
        .collect();
    list1.sort();
    assert_eq!(list1, generate_denoms("TOKEN", 0, 3));

    let mut list2: Vec<_> = suite
        .list_markets_with_pagination(list1.last().unwrap().clone(), 3)
        .unwrap()
        .markets
        .into_iter()
        .map(|r| r.base_asset)
        .collect();
    list2.sort();
    assert_eq!(list2, generate_denoms("TOKEN", 3, 5));
}
