import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { CrateTokenIDL } from "../idls/crate_token";

export * from "../idls/crate_token";

type CrateTokenTypes = AnchorTypes<
  CrateTokenIDL,
  {
    crateToken: CrateTokenData;
  }
>;

export type CrateTokenData = CrateTokenTypes["Accounts"]["CrateToken"];

export type CrateTokenProgram = CrateTokenTypes["Program"];
