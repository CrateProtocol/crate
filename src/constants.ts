import { buildCoderMap } from "@saberhq/anchor-contrib";
import { PublicKey } from "@solana/web3.js";

import type {
  CrateRedeemInKindProgram,
  CrateRedeemInKindTypes,
} from "./programs/crateRedeemInKind";
import { CrateRedeemInKindJSON } from "./programs/crateRedeemInKind";
import type { CrateTokenProgram, CrateTokenTypes } from "./programs/crateToken";
import { CrateTokenJSON } from "./programs/crateToken";

export const CRATE_IDLS = {
  CrateToken: CrateTokenJSON,
  CrateRedeemInKind: CrateRedeemInKindJSON,
};

export const CRATE_ADDRESSES = {
  CrateToken: new PublicKey("CRATwLpu6YZEeiVq9ajjxs61wPQ9f29s1UoQR9siJCRs"),
  CrateRedeemInKind: new PublicKey(
    "1NKyU3qShZC3oJgvCCftAHDi5TFxcJwfyUz2FeZsiwE"
  ),
};

export interface CratePrograms {
  CrateToken: CrateTokenProgram;
  CrateRedeemInKind: CrateRedeemInKindProgram;
}

export type CrateAddresses = { [K in keyof typeof CRATE_ADDRESSES]: PublicKey };

export const CRATE_FEE_OWNER = new PublicKey(
  "AAqAKWdsUPepSgXf7Msbp1pQ7yCPgYkBvXmNfTFBGAqp"
);

export const CRATE_REDEEM_IN_KIND_WITHDRAW_AUTHORITY = new PublicKey(
  "2amCDqmgpQ2qkryLArCcYeX8DzyNqvjuy7yKq6hsonqF"
);

/**
 * Coders for Crate accounts and programs.
 */
export const CRATE_CODERS = buildCoderMap<{
  CrateToken: CrateTokenTypes;
  CrateRedeemInKind: CrateRedeemInKindTypes;
}>(CRATE_IDLS, CRATE_ADDRESSES);
