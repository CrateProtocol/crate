import type { AnchorTypes } from "@saberhq/anchor-contrib";

import type { CrateRedeemInKindIDL } from "../idls/crate_redeem_in_kind";

export * from "../idls/crate_redeem_in_kind";

export type CrateRedeemInKindTypes = AnchorTypes<CrateRedeemInKindIDL>;

export type CrateRedeemInKindProgram = CrateRedeemInKindTypes["Program"];
