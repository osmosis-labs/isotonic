use super::suite::SuiteBuilder;
use crate::error::ContractError;

#[test]
fn adjust_common_token() {
    let mut suite = SuiteBuilder::new().build();

    let old_common_token = suite.query_config().unwrap().common_token;
    let new_token = "new_token";
    assert_ne!(old_common_token, new_token);

    suite
        .adjust_common_token(suite.credit_agency().as_str(), new_token)
        .unwrap();
    assert_eq!(new_token, suite.query_config().unwrap().common_token);
}

#[test]
fn adjust_common_token_without_ca() {
    let mut suite = SuiteBuilder::new().build();

    let err = suite
        .adjust_common_token("not_credit_agency", "new_token")
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}
