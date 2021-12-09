pub mod controller;
pub mod suite;

use crate::msg::TokenInfoResponse;
use crate::ContractError;
use cosmwasm_std::Uint128;
use suite::{Suite, SuiteBuilder};

#[test]
fn fresh_queries() {
    let suite = SuiteBuilder::new()
        .with_name("Lendex")
        .with_symbol("LDX")
        .with_decimals(9)
        .build();
    let actor = "actor";
    let controller = suite.controller();
    let controller = controller.as_str();

    assert_eq!(
        suite.query_token_info().unwrap(),
        TokenInfoResponse {
            name: "Lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            total_supply: Uint128::zero(),
        }
    );

    assert_eq!(suite.query_balance(actor).unwrap(), Uint128::zero());
    assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
}

mod minting {
    use super::*;

    #[test]
    fn by_controller() {
        let mut suite = Suite::new();
        let lender = "lender";
        let controller = suite.controller();
        let controller = controller.as_str();

        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    }

    #[test]
    fn by_non_controller() {
        let mut suite = Suite::new();
        let lender = "lender";
        let minter = "minter";
        let controller = suite.controller();
        let controller = controller.as_str();

        let err = suite.mint(minter, lender, Uint128::new(100)).unwrap_err();

        assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(0));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
        assert_eq!(suite.query_balance(minter).unwrap(), Uint128::zero());
    }
}

mod burning {
    use super::*;

    #[test]
    fn by_controller() {
        let mut suite = Suite::new();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to burn
        suite
            .mint(controller, controller, Uint128::new(100))
            .unwrap();

        // Actually burning
        suite.burn(controller, Uint128::new(50)).unwrap();

        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::new(50));
    }

    #[test]
    fn overflow() {
        let mut suite = Suite::new();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to burn
        suite
            .mint(controller, controller, Uint128::new(100))
            .unwrap();

        // Actually burning
        let err = suite.burn(controller, Uint128::new(150)).unwrap_err();

        assert_eq!(
            ContractError::InsufficientTokens {
                available: Uint128::new(100),
                needed: Uint128::new(150)
            },
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::new(100));
    }

    #[test]
    fn by_non_controller() {
        let mut suite = Suite::new();
        let lender = "lender";
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to burn
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        // Actually burning
        let err = suite.burn(lender, Uint128::new(150)).unwrap_err();

        assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    }
}

mod transfer {
    use super::*;

    #[test]
    fn proper() {
        let lender = "lender";
        let receiver = "receiver";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        suite.transfer(lender, receiver, Uint128::new(40)).unwrap();

        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(60));
        assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::new(40));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    }

    #[test]
    fn overflow() {
        let lender = "lender";
        let receiver = "receiver";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(200))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        let err = suite
            .transfer(lender, receiver, Uint128::new(140))
            .unwrap_err();

        assert_eq!(
            ContractError::InsufficientTokens {
                available: Uint128::new(100),
                needed: Uint128::new(140)
            },
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(100));
        assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::new(0));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    }

    #[test]
    fn not_enought_transferable() {
        let lender = "lender";
        let receiver = "receiver";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(200)).unwrap();

        let err = suite
            .transfer(lender, receiver, Uint128::new(140))
            .unwrap_err();

        assert_eq!(
            ContractError::CannotTransfer {
                max_transferable: Uint128::new(100),
            },
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(200));
        assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::new(0));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    }

    #[test]
    fn no_transferable() {
        let lender = "lender";
        let receiver = "receiver";
        let mut suite = Suite::new();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(200)).unwrap();

        let err = suite
            .transfer(lender, receiver, Uint128::new(140))
            .unwrap_err();

        assert_eq!(
            ContractError::CannotTransfer {
                max_transferable: Uint128::new(0),
            },
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_balance(lender).unwrap(), Uint128::new(200));
        assert_eq!(suite.query_balance(receiver).unwrap(), Uint128::new(0));
        assert_eq!(suite.query_balance(controller).unwrap(), Uint128::zero());
    }
}
