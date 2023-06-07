# Changelog

## [Unreleased](https://github.com/osmosis-labs/isotonic/tree/HEAD)

[Full Changelog](https://github.com/osmosis-labs/isotonic/compare/v0.6.0...HEAD)

## [v0.6.0](https://github.com/osmosis-labs/isotonic/tree/v0.6.0) (2023-06-07)

[Full Changelog](https://github.com/osmosis-labs/isotonic/compare/v0.5.0...v0.6.0)

**Fixed bugs:**

- user cannot withdraw amount listed on the screen [\#125](https://github.com/osmosis-labs/isotonic/issues/125)
- Market: Credit line should include existing debt [\#122](https://github.com/osmosis-labs/isotonic/issues/122)
- Add missing `entry_point` wrapper to market [\#114](https://github.com/osmosis-labs/isotonic/issues/114)
- Compounding might be broken [\#102](https://github.com/osmosis-labs/isotonic/issues/102)

**Closed issues:**

- Update Rust edition to 2021 [\#143](https://github.com/osmosis-labs/isotonic/issues/143)
- \[market\] SwapWithdrawFrom - optimize edge cases for lower fees [\#142](https://github.com/osmosis-labs/isotonic/issues/142)
- \[CA, Market\] Update osmosis-bindings-test to support swaps with a non-empty route [\#127](https://github.com/osmosis-labs/isotonic/issues/127)
- Market: Use proper collateral in `credit_line` query [\#121](https://github.com/osmosis-labs/isotonic/issues/121)
- Fix Rounding [\#120](https://github.com/osmosis-labs/isotonic/issues/120)
- Query: APY endpoint [\#117](https://github.com/osmosis-labs/isotonic/issues/117)
- Find a planning poker tool [\#116](https://github.com/osmosis-labs/isotonic/issues/116)
- Give CircleCI access to osmosis-bindings [\#113](https://github.com/osmosis-labs/isotonic/issues/113)
- Cut v0.5.0 release [\#106](https://github.com/osmosis-labs/isotonic/issues/106)
- Repay loans using collateral [\#101](https://github.com/osmosis-labs/isotonic/issues/101)
- Lending whitelist for markets [\#100](https://github.com/osmosis-labs/isotonic/issues/100)
- Liquidate via AMM [\#92](https://github.com/osmosis-labs/isotonic/issues/92)
- OSMOSIS bindings-test for mocking OSMOSIS in test suites [\#84](https://github.com/osmosis-labs/isotonic/issues/84)
- OSMOSIS bindings [\#83](https://github.com/osmosis-labs/isotonic/issues/83)
- Acceptance tests [\#71](https://github.com/osmosis-labs/isotonic/issues/71)
- Define APIs to accept cw20 and native [\#69](https://github.com/osmosis-labs/isotonic/issues/69)
- Real price oracle drawing from AMM price feeds  [\#67](https://github.com/osmosis-labs/isotonic/issues/67)
- Simulate interest payments when querying balances [\#56](https://github.com/osmosis-labs/isotonic/issues/56)

**Merged pull requests:**

- Update to latest osmosis-bindings 0.6.0 [\#152](https://github.com/osmosis-labs/isotonic/pull/152) ([maurolacy](https://github.com/maurolacy))
- Clarify license in README [\#150](https://github.com/osmosis-labs/isotonic/pull/150) ([ethanfrey](https://github.com/ethanfrey))
- Place everything under MIT license [\#149](https://github.com/osmosis-labs/isotonic/pull/149) ([ethanfrey](https://github.com/ethanfrey))
- \[market\] SwapWithdrawFrom - optimize edge cases [\#147](https://github.com/osmosis-labs/isotonic/pull/147) ([ueco-jb](https://github.com/ueco-jb))
- `swap_withdraw_from`/`repay_with_collateral` - more mutlitest cases  [\#146](https://github.com/osmosis-labs/isotonic/pull/146) ([ueco-jb](https://github.com/ueco-jb))
- Liquidate via AMM [\#145](https://github.com/osmosis-labs/isotonic/pull/145) ([uint](https://github.com/uint))
- Update Rust edition to 2021 [\#144](https://github.com/osmosis-labs/isotonic/pull/144) ([ueco-jb](https://github.com/ueco-jb))
- use osmo-bindings from crates.io, not github [\#141](https://github.com/osmosis-labs/isotonic/pull/141) ([uint](https://github.com/uint))
- Acceptance/regression test crate [\#140](https://github.com/osmosis-labs/isotonic/pull/140) ([uint](https://github.com/uint))
- The last of rounding issues [\#138](https://github.com/osmosis-labs/isotonic/pull/138) ([uint](https://github.com/uint))
- Update osmo-bindings [\#137](https://github.com/osmosis-labs/isotonic/pull/137) ([uint](https://github.com/uint))
- market: make doc comments more consistent [\#136](https://github.com/osmosis-labs/isotonic/pull/136) ([uint](https://github.com/uint))
- market: borrowable query [\#134](https://github.com/osmosis-labs/isotonic/pull/134) ([uint](https://github.com/uint))
- market: withdrawable query [\#133](https://github.com/osmosis-labs/isotonic/pull/133) ([uint](https://github.com/uint))
- Oracle: remove unused contract [\#131](https://github.com/osmosis-labs/isotonic/pull/131) ([ueco-jb](https://github.com/ueco-jb))
- \[CA, Market\] Use Osmosis Oracle [\#130](https://github.com/osmosis-labs/isotonic/pull/130) ([ueco-jb](https://github.com/ueco-jb))
- Osmosis Oracle: add `QueryMsg::PoolId` [\#129](https://github.com/osmosis-labs/isotonic/pull/129) ([ueco-jb](https://github.com/ueco-jb))
- Rounding improvements [\#126](https://github.com/osmosis-labs/isotonic/pull/126) ([hashedone](https://github.com/hashedone))
- Repay loans using collateral [\#124](https://github.com/osmosis-labs/isotonic/pull/124) ([ueco-jb](https://github.com/ueco-jb))
- isotonic-market: APY query [\#123](https://github.com/osmosis-labs/isotonic/pull/123) ([hashedone](https://github.com/hashedone))
- Update issue templates [\#119](https://github.com/osmosis-labs/isotonic/pull/119) ([ueco-jb](https://github.com/ueco-jb))
- Market: Add missing entry\_point decorator do query handler [\#115](https://github.com/osmosis-labs/isotonic/pull/115) ([ueco-jb](https://github.com/ueco-jb))
- Osmosis DEX price oracle [\#112](https://github.com/osmosis-labs/isotonic/pull/112) ([uint](https://github.com/uint))
- market: handle compound interest properly [\#109](https://github.com/osmosis-labs/isotonic/pull/109) ([uint](https://github.com/uint))
- All tokens replaced with being either native or cw20 [\#108](https://github.com/osmosis-labs/isotonic/pull/108) ([hashedone](https://github.com/hashedone))
- Simulate interest payments on query [\#103](https://github.com/osmosis-labs/isotonic/pull/103) ([uint](https://github.com/uint))

## [v0.5.0](https://github.com/confio/isotonic/tree/v0.5.0) (2022-03-08)

[Full Changelog](https://github.com/confio/isotonic/compare/v0.4.0...v0.5.0)

**Closed issues:**

- Market: Add multitests for sudo messages [\#93](https://github.com/confio/isotonic/issues/93)
- Rename to Isotonic [\#80](https://github.com/confio/isotonic/issues/80)
- Add separate message for charging interest without repay [\#57](https://github.com/confio/isotonic/issues/57)
- Allow governance to adjust credit agency [\#49](https://github.com/confio/isotonic/issues/49)
- Allow governance to adjust markets [\#48](https://github.com/confio/isotonic/issues/48)
- Update READMEs [\#36](https://github.com/confio/isotonic/issues/36)
- Optimize: Exit Markets  [\#25](https://github.com/confio/isotonic/issues/25)
- Optimize: Explicitly Enter markets [\#24](https://github.com/confio/isotonic/issues/24)

**Merged pull requests:**

- Rename repository to isotonic [\#105](https://github.com/confio/isotonic/pull/105) ([ueco-jb](https://github.com/ueco-jb))
- Add multitests for sudo messages [\#99](https://github.com/confio/isotonic/pull/99) ([ueco-jb](https://github.com/ueco-jb))
- Exitin markets [\#97](https://github.com/confio/isotonic/pull/97) ([hashedone](https://github.com/hashedone))
- Allow governance to adjust credit agency [\#96](https://github.com/confio/isotonic/pull/96) ([ueco-jb](https://github.com/ueco-jb))
- Update READMEs [\#94](https://github.com/confio/isotonic/pull/94) ([uint](https://github.com/uint))
- Optimization: Entering markets [\#91](https://github.com/confio/isotonic/pull/91) ([uint](https://github.com/uint))
- Allow governance to adjust markets [\#90](https://github.com/confio/isotonic/pull/90) ([ueco-jb](https://github.com/ueco-jb))

## [v0.4.0](https://github.com/confio/isotonic/tree/v0.4.0) (2022-03-02)

[Full Changelog](https://github.com/confio/isotonic/compare/v0.3.0...v0.4.0)

**Breaking changes:**

- Add reserve for each Market [\#47](https://github.com/confio/isotonic/issues/47)

**Closed issues:**

- New liquidation \(option 2\): each market maintains a list of opt-in liquidators [\#82](https://github.com/confio/isotonic/issues/82)
- Rename to Isotonic [\#80](https://github.com/confio/isotonic/issues/80)
- Update to cw-plus 0.12.1 [\#77](https://github.com/confio/isotonic/issues/77)
- Replace Market's `Uint128` responses with `Coin` to acknowledge denoms [\#73](https://github.com/confio/isotonic/issues/73)
- Liquidate via AMM [\#68](https://github.com/confio/isotonic/issues/68)
- Allow list for who can liquidate [\#66](https://github.com/confio/isotonic/issues/66)
- Add cap to each market [\#65](https://github.com/confio/isotonic/issues/65)
- Add new interest rate model [\#64](https://github.com/confio/isotonic/issues/64)

**Merged pull requests:**

- Add reserve factor for each market [\#85](https://github.com/confio/isotonic/pull/85) ([ueco-jb](https://github.com/ueco-jb))
- Optional market caps [\#81](https://github.com/confio/isotonic/pull/81) ([uint](https://github.com/uint))
- Add denoms to CreditLineResponse [\#79](https://github.com/confio/isotonic/pull/79) ([uint](https://github.com/uint))
- Update cw-plus packages to 0.12.1 [\#78](https://github.com/confio/isotonic/pull/78) ([ueco-jb](https://github.com/ueco-jb))
- Update changelog accordingly to 0.3.0 release [\#76](https://github.com/confio/isotonic/pull/76) ([ueco-jb](https://github.com/ueco-jb))
- Validate interest rate [\#74](https://github.com/confio/isotonic/pull/74) ([uint](https://github.com/uint))
- Piecewise linear interest rate model [\#72](https://github.com/confio/isotonic/pull/72) ([uint](https://github.com/uint))

## [v0.3.0](https://github.com/confio/isotonic/tree/v0.3.0) (2022-02-14)

[Full Changelog](https://github.com/confio/isotonic/compare/v0.2.0...v0.3.0)

**Closed issues:**

- Price Oracle: Return inverse rate if buy/sell denoms matches [\#45](https://github.com/confio/isotonic/issues/45)
- Fix naming for price ratio [\#58](https://github.com/confio/isotonic/issues/58)
- Market: `collateral_ratio` must be lower then `liquidation_price` from CA [\#55](https://github.com/confio/isotonic/issues/55)
- Market Contract - Credit Agency authorizes transferable amount [\#53](https://github.com/confio/isotonic/issues/53)
- Set version `0.3.0` [\#50](https://github.com/confio/isotonic/issues/50)
- Direct Liquidation Possible [\#27](https://github.com/confio/isotonic/issues/27)
- Credit Agency properly authorizes borrows and transfers [\#26](https://github.com/confio/isotonic/issues/26)
- Add query for total credit on Credit Agency [\#23](https://github.com/confio/isotonic/issues/23)
- Provide Collateral Info from Market [\#21](https://github.com/confio/isotonic/issues/21)
- Create \(stub\) Price Oracle [\#20](https://github.com/confio/isotonic/issues/20)
- Create Credit Agency as Market Factory [\#19](https://github.com/confio/isotonic/issues/19)

**Merged pull requests:**

- Change contract's version to 0.3.0 [\#75](https://github.com/confio/isotonic/pull/75) ([ueco-jb](https://github.com/ueco-jb))
- Update workspace-optimizer to latest v0.12.4 [\#62](https://github.com/confio/isotonic/pull/62) ([maurolacy](https://github.com/maurolacy))
- Fix tag consolidation for matching CHANGELOG entries [\#61](https://github.com/confio/isotonic/pull/61) ([maurolacy](https://github.com/maurolacy))
- CA: Return error if liquidation price is lower then collateral ratio [\#60](https://github.com/confio/isotonic/pull/60) ([ueco-jb](https://github.com/ueco-jb))
- Market: Price ratio helper refactored [\#59](https://github.com/confio/isotonic/pull/59) ([ueco-jb](https://github.com/ueco-jb))
- Market contract: CA authorizes transferable amount [\#54](https://github.com/confio/isotonic/pull/54) ([ueco-jb](https://github.com/ueco-jb))
- Credit Agency: Direct liquidation [\#52](https://github.com/confio/isotonic/pull/52) ([ueco-jb](https://github.com/ueco-jb))
- CA authorizes borrows and transfers [\#51](https://github.com/confio/isotonic/pull/51) ([ueco-jb](https://github.com/ueco-jb))
- Credit Agency - add query for total credit [\#46](https://github.com/confio/isotonic/pull/46) ([ueco-jb](https://github.com/ueco-jb))
- Rename `base_asset` to `market_token` [\#44](https://github.com/confio/isotonic/pull/44) ([ueco-jb](https://github.com/ueco-jb))
- Provide collateral info from market [\#41](https://github.com/confio/isotonic/pull/41) ([ueco-jb](https://github.com/ueco-jb))

## [Unreleased](https://github.com/confio/isotonic/tree/HEAD)

[Full Changelog](https://github.com/confio/isotonic/compare/v0.1.0...HEAD)

## [v0.2.0](https://github.com/confio/isotonic/tree/v0.2.0) (2021-12-20)

[Full Changelog](https://github.com/confio/isotonic/compare/v0.1.0...v0.2.0)

**Implemented enhancements:**

-  Remove schemas, and publish them with artifacts on release tags [\#33](https://github.com/confio/isotonic/issues/33)

**Closed issues:**

- Create \(stub\) Price Oracle [\#20](https://github.com/confio/isotonic/issues/20)
- Create Credit Agency as Market Factory [\#19](https://github.com/confio/isotonic/issues/19)
- Charge Interest [\#9](https://github.com/confio/isotonic/issues/9)
- Calculate Interest and Utilisation rate [\#8](https://github.com/confio/isotonic/issues/8)
- Borrow and Repay B Tokens [\#7](https://github.com/confio/isotonic/issues/7)
- Mint/Burn L Tokens [\#6](https://github.com/confio/isotonic/issues/6)
- Instantiate Market contract [\#5](https://github.com/confio/isotonic/issues/5)
- Add cw2222 style Distribution [\#4](https://github.com/confio/isotonic/issues/4)

**Merged pull requests:**

- Isotonic Market: Move execute messages to separate mod [\#39](https://github.com/confio/isotonic/pull/39) ([ueco-jb](https://github.com/ueco-jb))
- Create Credit Agency contract [\#38](https://github.com/confio/isotonic/pull/38) ([uint](https://github.com/uint))
- isotonic-oracle: set up the oracle contract [\#37](https://github.com/confio/isotonic/pull/37) ([uint](https://github.com/uint))
- Isotonic Market - charge interests [\#35](https://github.com/confio/isotonic/pull/35) ([ueco-jb](https://github.com/ueco-jb))
- Remove scheams from contracts [\#34](https://github.com/confio/isotonic/pull/34) ([ueco-jb](https://github.com/ueco-jb))
- Market Contract - Borrow and repay BTokens [\#32](https://github.com/confio/isotonic/pull/32) ([ueco-jb](https://github.com/ueco-jb))
- Interest and utilisation rates [\#31](https://github.com/confio/isotonic/pull/31) ([maurolacy](https://github.com/maurolacy))
- isotonic-token: Cw2222 style distribution [\#30](https://github.com/confio/isotonic/pull/30) ([hashedone](https://github.com/hashedone))
- isotonic-token: controller can burn anyone's coin [\#29](https://github.com/confio/isotonic/pull/29) ([uint](https://github.com/uint))
- isotonic-market: Mint/burn l-tokens [\#28](https://github.com/confio/isotonic/pull/28) ([uint](https://github.com/uint))

## [v0.1.0](https://github.com/confio/isotonic/tree/v0.1.0) (2021-12-10)

[Full Changelog](https://github.com/confio/isotonic/compare/849f1119e4f6e371421b90a1d667feb18f84e396...v0.1.0)

**Closed issues:**

- Set up CI config [\#12](https://github.com/confio/isotonic/issues/12)
- Provide one contract \(straight from cw-template\) called `isotonic-token`. [\#11](https://github.com/confio/isotonic/issues/11)
- Add rebasing to isotonic-token [\#3](https://github.com/confio/isotonic/issues/3)
- Minimal Isotonic Token \(no rebasing\) [\#2](https://github.com/confio/isotonic/issues/2)
- Set up repo [\#1](https://github.com/confio/isotonic/issues/1)

**Merged pull requests:**

- Basic isotonic contract [\#18](https://github.com/confio/isotonic/pull/18) ([hashedone](https://github.com/hashedone))
- isotonic-token: rebasing [\#17](https://github.com/confio/isotonic/pull/17) ([uint](https://github.com/uint))
- Instantiate Market contract [\#16](https://github.com/confio/isotonic/pull/16) ([ueco-jb](https://github.com/ueco-jb))
- Create new Market contract [\#15](https://github.com/confio/isotonic/pull/15) ([ueco-jb](https://github.com/ueco-jb))
- isotonic-token: Basic contract [\#14](https://github.com/confio/isotonic/pull/14) ([hashedone](https://github.com/hashedone))
- Setup CI config [\#13](https://github.com/confio/isotonic/pull/13) ([ueco-jb](https://github.com/ueco-jb))
- Add isotonic-token contract base and empty package `utils` [\#10](https://github.com/confio/isotonic/pull/10) ([ueco-jb](https://github.com/ueco-jb))



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
