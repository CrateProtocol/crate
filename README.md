# 📦 Crate Protocol

[![License](https://img.shields.io/badge/license-AGPL%203.0-blue)](https://github.com/CrateProtocol/crate/blob/master/LICENSE)
[![Build Status](https://img.shields.io/github/workflow/status/CrateProtocol/crate/E2E/master)](https://github.com/CrateProtocol/crate/actions/workflows/programs-e2e.yml?query=branch%3Amaster)
[![Contributors](https://img.shields.io/github/contributors/CrateProtocol/crate)](https://github.com/CrateProtocol/crate/graphs/contributors)

![Crate Protocol](/images/banner.png)

Crate Protocol allows anyone to create, manage, and trade a tokenized basket of assets, which we refer to as a **Crate**. A Crate is always fully collateralized by its underlying assets. The protocol will evolve to support advanced features, including automatic rebalancing based on set parameters.

We're in active development. For the latest updates, please join our community:

- Twitter: https://twitter.com/CrateProtocol
- Discord: https://chat.crate.so

## Note

- **Crate is in active development, so all APIs are subject to change.**
- **This code is unaudited. Use at your own risk.**

## Packages

| Package                    | Description                                          | Version                                                                                                                     | Docs                                                                                 |
| :------------------------- | :--------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------- |
| `crate-token`              | Fractional ownership of a basket of fungible assets. | [![Crates.io](https://img.shields.io/crates/v/crate-token)](https://crates.io/crates/crate-token)                           | [![Docs.rs](https://docs.rs/crate-token/badge.svg)](https://docs.rs/crate-token)     |
| `@crateprotocol/crate-sdk` | TypeScript SDK for Crate                             | [![npm](https://img.shields.io/npm/v/@crateprotocol/crate-sdk.svg)](https://www.npmjs.com/package/@crateprotocol/crate-sdk) | [![Docs](https://img.shields.io/badge/docs-typedoc-blue)](https://docs.crate.so/ts/) |

## Addresses

Program addresses are the same on devnet, testnet, and mainnet-beta.

- Crate: [`CRAToQ6Ycrxp6ei3VBcdi8oRxETkwDJZxRu6ziCFGD5a`](https://explorer.solana.com/address/CRAToQ6Ycrxp6ei3VBcdi8oRxETkwDJZxRu6ziCFGD5a)

## Contribution

Thank you for your interest in contributing to Crate Protocol! All contributions are welcome no matter how big or small. This includes (but is not limited to) filing issues, adding documentation, fixing bugs, creating examples, and implementing features.

When contributing, please make sure your code adheres to some basic coding guidlines:

- Code must be formatted with the configured formatters (e.g. rustfmt and prettier).
- Comment lines should be no longer than 80 characters and written with proper grammar and punctuation.
- Commit messages should be prefixed with the package(s) they modify. Changes affecting multiple packages should list all packages. In rare cases, changes may omit the package name prefix.

## License

Crate Protocol is licensed under the AGPL-3.0 license.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Crate Protocol by you, as defined in the AGPL-3.0 license, shall be licensed as above, without any additional terms or conditions.
