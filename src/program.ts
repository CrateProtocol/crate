import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { CrateTokenIDL } from "./idls/crate_token";

export * from "./idls/crate_token";

type CrateTokenTypes = AnchorTypes<CrateTokenIDL>;

export type CrateInfoData = CrateTokenTypes["Accounts"]["CrateInfo"];

export type CrateTokenProgram = CrateTokenTypes["Program"];
