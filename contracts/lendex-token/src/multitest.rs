pub mod controller;
pub mod rebasing;
pub mod receiver;
pub mod suite;

use crate::display_amount::DisplayAmount;
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
            total_supply: DisplayAmount::zero(),
        }
    );

    assert_eq!(suite.query_balance(actor).unwrap(), DisplayAmount::zero());
    assert_eq!(
        suite.query_balance(controller).unwrap(),
        DisplayAmount::zero()
    );
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

        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn zero_amount() {
        let mut suite = Suite::new();
        let lender = "lender";
        let controller = suite.controller();
        let controller = controller.as_str();

        let err = suite.mint(controller, lender, Uint128::zero()).unwrap_err();

        assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
        assert_eq!(suite.query_balance(lender).unwrap(), DisplayAmount::zero());
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
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
        assert_eq!(suite.query_balance(lender).unwrap(), DisplayAmount::zero());
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(suite.query_balance(minter).unwrap(), DisplayAmount::zero());
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

        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::raw(50u128)
        );
    }

    #[test]
    fn zero_amount() {
        let mut suite = Suite::new();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to burn
        suite
            .mint(controller, controller, Uint128::new(100))
            .unwrap();

        // Actually burning
        let err = suite.burn(controller, Uint128::zero()).unwrap_err();

        assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::raw(100u128)
        );
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
            ContractError::insufficient_tokens(100u128, 150u128),
            err.downcast().unwrap()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::raw(100u128)
        );
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
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
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

        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(60u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::raw(40u128)
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn zero_amount() {
        let lender = "lender";
        let receiver = "receiver";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        let err = suite
            .transfer(lender, receiver, Uint128::zero())
            .unwrap_err();

        assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
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
            ContractError::insufficient_tokens(100u128, 140u128),
            err.downcast().unwrap()
        );
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn not_enough_transferable() {
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
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(200u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
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
                max_transferable: Uint128::zero(),
            },
            err.downcast().unwrap()
        );
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(200u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }
}

mod send {
    use super::*;
    use crate::multitest::receiver::Cw20ExecMsg;
    use cosmwasm_std::to_binary;

    #[test]
    fn proper() {
        let lender = "lender";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();
        let receiver = suite.receiver();
        let receiver = receiver.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();

        suite
            .send(lender, receiver, Uint128::new(40), exec)
            .unwrap();

        assert_eq!(suite.query_receiver().unwrap(), 1);
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(60u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::raw(40u128)
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn zero_amount() {
        let lender = "lender";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();
        let receiver = suite.receiver();
        let receiver = receiver.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();

        let err = suite
            .send(lender, receiver, Uint128::zero(), exec)
            .unwrap_err();

        assert_eq!(ContractError::InvalidZeroAmount {}, err.downcast().unwrap());
        assert_eq!(suite.query_receiver().unwrap(), 0);
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn remote_call_fails() {
        let lender = "lender";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();
        let receiver = suite.receiver();
        let receiver = receiver.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        let exec = to_binary(&Cw20ExecMsg::Invalid {}).unwrap();

        let err = suite
            .send(lender, receiver, Uint128::new(40), exec)
            .unwrap_err();

        assert_eq!("Invalid message on receiver", err.to_string());
        assert_eq!(suite.query_receiver().unwrap(), 0);
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn overflow() {
        let lender = "lender";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(200))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();
        let receiver = suite.receiver();
        let receiver = receiver.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(100)).unwrap();

        let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();

        let err = suite
            .send(lender, receiver, Uint128::new(140), exec)
            .unwrap_err();

        assert_eq!(
            ContractError::insufficient_tokens(100u128, 140u128),
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_receiver().unwrap(), 0);
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(100u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn not_enough_transferable() {
        let lender = "lender";
        let mut suite = SuiteBuilder::new()
            .with_transferable(lender, Uint128::new(100))
            .build();
        let controller = suite.controller();
        let controller = controller.as_str();
        let receiver = suite.receiver();
        let receiver = receiver.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(200)).unwrap();

        let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();

        let err = suite
            .send(lender, receiver, Uint128::new(140), exec)
            .unwrap_err();

        assert_eq!(
            ContractError::CannotTransfer {
                max_transferable: Uint128::new(100),
            },
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_receiver().unwrap(), 0);
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(200u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }

    #[test]
    fn no_transferable() {
        let lender = "lender";
        let mut suite = Suite::new();
        let controller = suite.controller();
        let controller = controller.as_str();
        let receiver = suite.receiver();
        let receiver = receiver.as_str();

        // Preparation to have anything to transfer
        suite.mint(controller, lender, Uint128::new(200)).unwrap();

        let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();

        let err = suite
            .send(lender, receiver, Uint128::new(140), exec)
            .unwrap_err();

        assert_eq!(
            ContractError::CannotTransfer {
                max_transferable: Uint128::zero(),
            },
            err.downcast().unwrap()
        );
        assert_eq!(suite.query_receiver().unwrap(), 0);
        assert_eq!(
            suite.query_balance(lender).unwrap(),
            DisplayAmount::raw(200u128)
        );
        assert_eq!(
            suite.query_balance(receiver).unwrap(),
            DisplayAmount::zero()
        );
        assert_eq!(
            suite.query_balance(controller).unwrap(),
            DisplayAmount::zero()
        );
    }
}
