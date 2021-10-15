import { utils } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";

import { CRATE_TOKEN_PROGRAM_ID } from "./constants";

export const generateCrateAddress = (
  mint: PublicKey,
  programID: PublicKey = CRATE_TOKEN_PROGRAM_ID
): Promise<[PublicKey, number]> => {
  return PublicKey.findProgramAddress(
    [utils.bytes.utf8.encode("CrateInfo"), mint.toBuffer()],
    programID
  );
};
