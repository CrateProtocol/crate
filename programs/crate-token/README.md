# `crate-token`

[![Crates.io](https://img.shields.io/crates/v/crate-token)](https://crates.io/crates/crate-token)

Program which allows users to create a token that is redeemable for its underlying assets.

This can be used for many use cases, including but not limited to:

- ETFs
- Composable Stablecoins
- Rewards distributions

Program Address: [`CRATwLpu6YZEeiVq9ajjxs61wPQ9f29s1UoQR9siJCRs`](https://explorer.solana.com/address/CRATwLpu6YZEeiVq9ajjxs61wPQ9f29s1UoQR9siJCRs)

## Protocol fees

Protocol fees are taken from the `issue_fee` and the `withdraw_fee`-- 20% of this fee goes to the Crate DAO, while the other 80% goes to the Crate's "author". These fees are set by the `fee_setter` and default to zero.

There are no fees if the Crate's consumer does not take any fees. We want Crate to be a common building block for any ETF-like protocol on the Solana blockchain.
