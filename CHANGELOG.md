# Changelog

## [v0.3.0](https://github.com/confio/lendex/tree/v0.3.0) (2022-02-14)

[Full Changelog](https://github.com/confio/lendex/compare/v0.2.0...v0.3.0)

**Closed issues:**

- Price Oracle: Return inverse rate if buy/sell denoms matches [\#45](https://github.com/confio/lendex/issues/45)
- Fix naming for price ratio [\#58](https://github.com/confio/lendex/issues/58)
- Market: `collateral_ratio` must be lower then `liquidation_price` from CA [\#55](https://github.com/confio/lendex/issues/55)
- Market Contract - Credit Agency authorizes transferable amount [\#53](https://github.com/confio/lendex/issues/53)
- Set version `0.3.0` [\#50](https://github.com/confio/lendex/issues/50)
- Direct Liquidation Possible [\#27](https://github.com/confio/lendex/issues/27)
- Credit Agency properly authorizes borrows and transfers [\#26](https://github.com/confio/lendex/issues/26)
- Add query for total credit on Credit Agency [\#23](https://github.com/confio/lendex/issues/23)
- Provide Collateral Info from Market [\#21](https://github.com/confio/lendex/issues/21)
- Create \(stub\) Price Oracle [\#20](https://github.com/confio/lendex/issues/20)
- Create Credit Agency as Market Factory [\#19](https://github.com/confio/lendex/issues/19)

**Merged pull requests:**

- Change contract's version to 0.3.0 [\#75](https://github.com/confio/lendex/pull/75) ([ueco-jb](https://github.com/ueco-jb))
- Update workspace-optimizer to latest v0.12.4 [\#62](https://github.com/confio/lendex/pull/62) ([maurolacy](https://github.com/maurolacy))
- Fix tag consolidation for matching CHANGELOG entries [\#61](https://github.com/confio/lendex/pull/61) ([maurolacy](https://github.com/maurolacy))
- CA: Return error if liquidation price is lower then collateral ratio [\#60](https://github.com/confio/lendex/pull/60) ([ueco-jb](https://github.com/ueco-jb))
- Market: Price ratio helper refactored [\#59](https://github.com/confio/lendex/pull/59) ([ueco-jb](https://github.com/ueco-jb))
- Market contract: CA authorizes transferable amount [\#54](https://github.com/confio/lendex/pull/54) ([ueco-jb](https://github.com/ueco-jb))
- Credit Agency: Direct liquidation [\#52](https://github.com/confio/lendex/pull/52) ([ueco-jb](https://github.com/ueco-jb))
- CA authorizes borrows and transfers [\#51](https://github.com/confio/lendex/pull/51) ([ueco-jb](https://github.com/ueco-jb))
- Credit Agency - add query for total credit [\#46](https://github.com/confio/lendex/pull/46) ([ueco-jb](https://github.com/ueco-jb))
- Rename `base_asset` to `market_token` [\#44](https://github.com/confio/lendex/pull/44) ([ueco-jb](https://github.com/ueco-jb))
- Provide collateral info from market [\#41](https://github.com/confio/lendex/pull/41) ([ueco-jb](https://github.com/ueco-jb))

## [Unreleased](https://github.com/confio/lendex/tree/HEAD)

[Full Changelog](https://github.com/confio/lendex/compare/v0.1.0...HEAD)

## [v0.2.0](https://github.com/confio/lendex/tree/v0.2.0) (2021-12-20)

[Full Changelog](https://github.com/confio/lendex/compare/v0.1.0...v0.2.0)

**Implemented enhancements:**

-  Remove schemas, and publish them with artifacts on release tags [\#33](https://github.com/confio/lendex/issues/33)

**Closed issues:**

- Create \(stub\) Price Oracle [\#20](https://github.com/confio/lendex/issues/20)
- Create Credit Agency as Market Factory [\#19](https://github.com/confio/lendex/issues/19)
- Charge Interest [\#9](https://github.com/confio/lendex/issues/9)
- Calculate Interest and Utilisation rate [\#8](https://github.com/confio/lendex/issues/8)
- Borrow and Repay B Tokens [\#7](https://github.com/confio/lendex/issues/7)
- Mint/Burn L Tokens [\#6](https://github.com/confio/lendex/issues/6)
- Instantiate Market contract [\#5](https://github.com/confio/lendex/issues/5)
- Add cw2222 style Distribution [\#4](https://github.com/confio/lendex/issues/4)

**Merged pull requests:**

- Lendex Market: Move execute messages to separate mod [\#39](https://github.com/confio/lendex/pull/39) ([ueco-jb](https://github.com/ueco-jb))
- Create Credit Agency contract [\#38](https://github.com/confio/lendex/pull/38) ([uint](https://github.com/uint))
- lendex-oracle: set up the oracle contract [\#37](https://github.com/confio/lendex/pull/37) ([uint](https://github.com/uint))
- Lendex Market - charge interests [\#35](https://github.com/confio/lendex/pull/35) ([ueco-jb](https://github.com/ueco-jb))
- Remove scheams from contracts [\#34](https://github.com/confio/lendex/pull/34) ([ueco-jb](https://github.com/ueco-jb))
- Market Contract - Borrow and repay BTokens [\#32](https://github.com/confio/lendex/pull/32) ([ueco-jb](https://github.com/ueco-jb))
- Interest and utilisation rates [\#31](https://github.com/confio/lendex/pull/31) ([maurolacy](https://github.com/maurolacy))
- lendex-token: Cw2222 style distribution [\#30](https://github.com/confio/lendex/pull/30) ([hashedone](https://github.com/hashedone))
- lendex-token: controller can burn anyone's coin [\#29](https://github.com/confio/lendex/pull/29) ([uint](https://github.com/uint))
- lendex-market: Mint/burn l-tokens [\#28](https://github.com/confio/lendex/pull/28) ([uint](https://github.com/uint))

## [v0.1.0](https://github.com/confio/lendex/tree/v0.1.0) (2021-12-10)

[Full Changelog](https://github.com/confio/lendex/compare/849f1119e4f6e371421b90a1d667feb18f84e396...v0.1.0)

**Closed issues:**

- Set up CI config [\#12](https://github.com/confio/lendex/issues/12)
- Provide one contract \(straight from cw-template\) called `lendex-token`. [\#11](https://github.com/confio/lendex/issues/11)
- Add rebasing to lendex-token [\#3](https://github.com/confio/lendex/issues/3)
- Minimal Lendex Token \(no rebasing\) [\#2](https://github.com/confio/lendex/issues/2)
- Set up repo [\#1](https://github.com/confio/lendex/issues/1)

**Merged pull requests:**

- Basic lendex contract [\#18](https://github.com/confio/lendex/pull/18) ([hashedone](https://github.com/hashedone))
- lendex-token: rebasing [\#17](https://github.com/confio/lendex/pull/17) ([uint](https://github.com/uint))
- Instantiate Market contract [\#16](https://github.com/confio/lendex/pull/16) ([ueco-jb](https://github.com/ueco-jb))
- Create new Market contract [\#15](https://github.com/confio/lendex/pull/15) ([ueco-jb](https://github.com/ueco-jb))
- lendex-token: Basic contract [\#14](https://github.com/confio/lendex/pull/14) ([hashedone](https://github.com/hashedone))
- Setup CI config [\#13](https://github.com/confio/lendex/pull/13) ([ueco-jb](https://github.com/ueco-jb))
- Add lendex-token contract base and empty package `utils` [\#10](https://github.com/confio/lendex/pull/10) ([ueco-jb](https://github.com/ueco-jb))



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*
