import { PublicKey } from "@solana/web3.js";

export const CRATE_ADDRESSES = {
  CrateToken: new PublicKey("CRATwLpu6YZEeiVq9ajjxs61wPQ9f29s1UoQR9siJCRs"),
  CrateRedeemInKind: new PublicKey(
    "1NKyU3qShZC3oJgvCCftAHDi5TFxcJwfyUz2FeZsiwE"
  ),
};

export type Addresses = { [K in keyof typeof CRATE_ADDRESSES]: PublicKey };

export const CRATE_FEE_OWNER = new PublicKey(
  "AAqAKWdsUPepSgXf7Msbp1pQ7yCPgYkBvXmNfTFBGAqp"
);

export const CRATE_REDEEM_IN_KIND_WITHDRAW_AUTHORITY = new PublicKey(
  "2amCDqmgpQ2qkryLArCcYeX8DzyNqvjuy7yKq6hsonqF"
);
