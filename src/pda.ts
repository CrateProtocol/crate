import { utils } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";

import { CRATE_ADDRESSES } from "./constants";

export const generateCrateAddress = (
  mint: PublicKey,
  programID: PublicKey = CRATE_ADDRESSES.CrateToken
): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode("CrateToken"), mint.toBuffer()],
    programID
  );
};
