import { Program, Provider as AnchorProvider } from "@project-serum/anchor";
import type { Provider } from "@saberhq/solana-contrib";
import {
  SignerWallet,
  SolanaProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import type { Token, TokenAmount } from "@saberhq/token-utils";
import {
  createInitMintInstructions,
  getOrCreateATAs,
  TOKEN_PROGRAM_ID,
} from "@saberhq/token-utils";
import type { AccountMeta, PublicKey, Signer } from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";
import invariant from "tiny-invariant";

import { CRATE_FEE_OWNER, CRATE_TOKEN_PROGRAM_ID } from "./constants";
import { CrateTokenJSON } from "./idls/crate_token";
import { generateCrateAddress } from "./pda";
import type { CrateTokenProgram } from "./program";

/**
 * Javascript SDK for interacting with Crate tokens.
 */
export class CrateTokenSDK {
  constructor(
    public readonly provider: Provider,
    public readonly program: CrateTokenProgram
  ) {}

  /**
   * Initialize from a Provider
   * @param provider
   * @param crateTokenProgramId
   * @returns
   */
  static init(
    provider: Provider,
    crateTokenProgramId: PublicKey = CRATE_TOKEN_PROGRAM_ID
  ): CrateTokenSDK {
    return new CrateTokenSDK(
      provider,
      new Program(
        CrateTokenJSON,
        crateTokenProgramId,
        new AnchorProvider(provider.connection, provider.wallet, provider.opts)
      ) as unknown as CrateTokenProgram
    );
  }

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  public withSigner(signer: Signer): CrateTokenSDK {
    return CrateTokenSDK.init(
      new SolanaProvider(
        this.provider.connection,
        this.provider.broadcaster,
        new SignerWallet(signer),
        this.provider.opts
      )
    );
  }

  /**
   * Creates a new Crate.
   * @returns
   */
  async newCrate({
    mintKP = Keypair.generate(),
    decimals = 6,
    payer = this.provider.wallet.publicKey,
    issueAuthority = this.provider.wallet.publicKey,
  }: {
    mintKP?: Keypair;
    decimals?: number;
    payer?: PublicKey;
    issueAuthority?: PublicKey;
  } = {}): Promise<{ tx: TransactionEnvelope; crateKey: PublicKey }> {
    const [crateKey, bump] = await generateCrateAddress(mintKP.publicKey);

    const initMintTX = await createInitMintInstructions({
      provider: this.provider,
      mintKP,
      decimals,
      mintAuthority: crateKey,
      freezeAuthority: crateKey,
    });

    const newCrateTX = new TransactionEnvelope(this.provider, [
      this.program.instruction.newCrate(bump, {
        accounts: {
          crateInfo: crateKey,
          crateMint: mintKP.publicKey,
          payer,
          systemProgram: SystemProgram.programId,
          issueAuthority,
        },
      }),
    ]);

    return { tx: initMintTX.combine(newCrateTX), crateKey };
  }

  /**
   * Issues Crate tokens as the issuer.
   * @returns
   */
  async issue({
    amount,
    issueAuthority = this.provider.wallet.publicKey,
    mintDestination,
  }: {
    amount: TokenAmount;
    issueAuthority?: PublicKey;
    mintDestination: PublicKey;
  }): Promise<TransactionEnvelope> {
    const [crateKey] = await generateCrateAddress(amount.token.mintAccount);
    return new TransactionEnvelope(this.provider, [
      this.program.instruction.issue(amount.toU64(), {
        accounts: {
          crateInfo: crateKey,
          crateMint: amount.token.mintAccount,
          issueAuthority,
          mintDestination,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      }),
    ]);
  }

  /**
   * Redeems Crate tokens for the underlying tokens.
   */
  async redeem({
    amount,
    owner = this.provider.wallet.publicKey,
    underlyingTokens,
  }: {
    amount: TokenAmount;
    owner?: PublicKey;
    /**
     * Underlying tokens list. Ensure this is complete.
     */
    underlyingTokens: Token[];
  }): Promise<TransactionEnvelope> {
    const underlyingMints = underlyingTokens.reduce(
      (acc, tok) => ({ ...acc, [tok.address]: tok.mintAccount }),
      {}
    );
    const ownerATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        crate: amount.token.mintAccount,
        ...underlyingMints,
      },
      owner,
    });

    const [crateKey] = await generateCrateAddress(amount.token.mintAccount);

    const crateATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: underlyingMints,
      owner: crateKey,
    });

    const feeATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: underlyingMints,
      owner: CRATE_FEE_OWNER,
    });

    const remainingAccounts = underlyingTokens
      .flatMap((token) => {
        const crateATA = (crateATAs.accounts as Record<string, PublicKey>)[
          token.address
        ];
        const ownerATA = (ownerATAs.accounts as Record<string, PublicKey>)[
          token.address
        ];
        const feeATA = (feeATAs.accounts as Record<string, PublicKey>)[
          token.address
        ];
        invariant(ownerATA && crateATA && feeATA, "missing ATA");
        return [crateATA, ownerATA, feeATA];
      })
      .map(
        (acc): AccountMeta => ({
          pubkey: acc,
          isSigner: false,
          isWritable: true,
        })
      );

    const env = new TransactionEnvelope(this.provider, [
      this.program.instruction.redeem(amount.toU64(), {
        accounts: {
          crateInfo: crateKey,
          crateMint: amount.token.mintAccount,
          crateSource: ownerATAs.accounts.crate,
          owner,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        remainingAccounts,
      }),
    ]);
    env.instructions.unshift(...ownerATAs.instructions);
    return env;
  }
}
