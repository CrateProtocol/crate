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
  getOrCreateATA,
  getOrCreateATAs,
  TOKEN_PROGRAM_ID,
} from "@saberhq/token-utils";
import type {
  AccountMeta,
  Signer,
  TransactionInstruction,
} from "@solana/web3.js";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import invariant from "tiny-invariant";

import type { Addresses } from "./constants";
import {
  CRATE_ADDRESSES,
  CRATE_FEE_OWNER,
  CRATE_REDEEM_IN_KIND_WITHDRAW_AUTHORITY,
} from "./constants";
import { CrateRedeemInKindJSON } from "./idls/crate_redeem_in_kind";
import { CrateTokenJSON } from "./idls/crate_token";
import { generateCrateAddress } from "./pda";
import type { CrateRedeemInKindProgram } from "./programs/crateRedeemInKind";
import type { CrateTokenData, CrateTokenProgram } from "./programs/crateToken";

/**
 * Javascript SDK for interacting with the Crate protocol.
 */
export class CrateSDK {
  constructor(
    public readonly provider: Provider,
    public readonly programs: {
      CrateToken: CrateTokenProgram;
      CrateRedeemInKind: CrateRedeemInKindProgram;
    }
  ) {}

  /**
   * Initialize from a Provider
   * @param provider
   * @param crateTokenProgramId
   * @returns
   */
  static init(
    provider: Provider,
    addresses: Addresses = CRATE_ADDRESSES
  ): CrateSDK {
    return new CrateSDK(provider, {
      CrateToken: new Program(
        CrateTokenJSON,
        addresses.CrateToken,
        new AnchorProvider(provider.connection, provider.wallet, provider.opts)
      ) as unknown as CrateTokenProgram,
      CrateRedeemInKind: new Program(
        CrateRedeemInKindJSON,
        addresses.CrateRedeemInKind,
        new AnchorProvider(provider.connection, provider.wallet, provider.opts)
      ) as unknown as CrateRedeemInKindProgram,
    });
  }

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  public withSigner(signer: Signer): CrateSDK {
    return CrateSDK.init(
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

    feeToSetter = this.provider.wallet.publicKey,
    feeSetterAuthority = this.provider.wallet.publicKey,
    issueAuthority = this.provider.wallet.publicKey,
    withdrawAuthority = CRATE_REDEEM_IN_KIND_WITHDRAW_AUTHORITY,
    authorFeeTo = PublicKey.default,
  }: {
    mintKP?: Keypair;
    decimals?: number;
    payer?: PublicKey;

    feeToSetter?: PublicKey;
    feeSetterAuthority?: PublicKey;
    issueAuthority?: PublicKey;
    withdrawAuthority?: PublicKey;
    /**
     * Who to send the author fees to.
     */
    authorFeeTo?: PublicKey;
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
      this.programs.CrateToken.instruction.newCrate(bump, {
        accounts: {
          crateToken: crateKey,
          crateMint: mintKP.publicKey,
          feeToSetter,
          feeSetterAuthority,
          issueAuthority,
          withdrawAuthority,
          authorFeeTo,
          payer,
          systemProgram: SystemProgram.programId,
        },
      }),
    ]);

    return { tx: initMintTX.combine(newCrateTX), crateKey };
  }

  setIssueFee(crateKey: PublicKey, feeBPS: number): TransactionEnvelope {
    return new TransactionEnvelope(this.provider, [
      this.programs.CrateToken.instruction.setIssueFee(feeBPS, {
        accounts: {
          crateToken: crateKey,
          feeSetter: this.provider.wallet.publicKey,
        },
      }),
    ]);
  }

  setWithdrawFee(crateKey: PublicKey, feeBPS: number): TransactionEnvelope {
    return new TransactionEnvelope(this.provider, [
      this.programs.CrateToken.instruction.setWithdrawFee(feeBPS, {
        accounts: {
          crateToken: crateKey,
          feeSetter: this.provider.wallet.publicKey,
        },
      }),
    ]);
  }

  async fetchCrateTokenData(key: PublicKey): Promise<CrateTokenData | null> {
    return (await this.programs.CrateToken.account.crateToken.fetchNullable(
      key
    )) as CrateTokenData;
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

    const crateTokenData = await this.fetchCrateTokenData(crateKey);
    if (!crateTokenData) {
      throw new Error("Crate does not exist.");
    }

    const ixs = [];
    const feeDestinations = {
      authorFeeDestination: mintDestination,
      protocolFeeDestination: mintDestination,
    };
    if (crateTokenData.issueFeeBps !== 0) {
      const [authorFeeATA, protocolFeeATA] = await Promise.all([
        getOrCreateATA({
          provider: this.provider,
          mint: crateTokenData.mint,
          owner: crateTokenData.authorFeeTo,
        }),
        getOrCreateATA({
          provider: this.provider,
          mint: crateTokenData.mint,
          owner: CRATE_FEE_OWNER,
        }),
      ]);

      feeDestinations.authorFeeDestination = authorFeeATA.address;
      feeDestinations.protocolFeeDestination = protocolFeeATA.address;
      ixs.push(
        ...[
          ...(authorFeeATA.instruction ? [authorFeeATA.instruction] : []),
          ...(protocolFeeATA.instruction ? [protocolFeeATA.instruction] : []),
        ]
      );
    }

    return new TransactionEnvelope(this.provider, [
      this.programs.CrateToken.instruction.issue(amount.toU64(), {
        accounts: {
          crateToken: crateKey,
          crateMint: amount.token.mintAccount,
          issueAuthority,
          mintDestination,
          tokenProgram: TOKEN_PROGRAM_ID,
          ...feeDestinations,
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
    const [crateKey] = await generateCrateAddress(amount.token.mintAccount);
    const crateTokenData =
      (await this.programs.CrateToken.account.crateToken.fetchNullable(
        crateKey
      )) as CrateTokenData;
    if (!crateTokenData) {
      throw new Error("Crate not found.");
    }

    if (
      !crateTokenData.withdrawAuthority.equals(
        CRATE_REDEEM_IN_KIND_WITHDRAW_AUTHORITY
      )
    ) {
      throw new Error("Expected REDEEM_IN_KIND withdraw authority.");
    }

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

    const crateATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: underlyingMints,
      owner: crateKey,
    });

    const additionalInstructions: TransactionInstruction[] = [];
    const remainingAccountKeys = await (async (): Promise<PublicKey[]> => {
      if (crateTokenData.withdrawFeeBps !== 0) {
        const authorFeeATAs = await getOrCreateATAs({
          provider: this.provider,
          mints: underlyingMints,
          owner: crateTokenData.authorFeeTo,
        });
        additionalInstructions.push(...authorFeeATAs.instructions);

        const protocolFeeATAs = await getOrCreateATAs({
          provider: this.provider,
          mints: underlyingMints,
          owner: CRATE_FEE_OWNER,
        });
        additionalInstructions.push(...protocolFeeATAs.instructions);

        return underlyingTokens.flatMap((token) => {
          const crateATA = (crateATAs.accounts as Record<string, PublicKey>)[
            token.address
          ];
          const ownerATA = (ownerATAs.accounts as Record<string, PublicKey>)[
            token.address
          ];
          const authorFeeATA = (
            authorFeeATAs.accounts as Record<string, PublicKey>
          )[token.address];
          const protocolFeeATA = (
            protocolFeeATAs.accounts as Record<string, PublicKey>
          )[token.address];
          invariant(
            ownerATA && crateATA && authorFeeATA && protocolFeeATA,
            "missing ATA"
          );
          return [crateATA, ownerATA, authorFeeATA, protocolFeeATA];
        });
      } else {
        return underlyingTokens.flatMap((token) => {
          const crateATA = (crateATAs.accounts as Record<string, PublicKey>)[
            token.address
          ];
          const ownerATA = (ownerATAs.accounts as Record<string, PublicKey>)[
            token.address
          ];
          invariant(ownerATA && crateATA, "missing ATA");
          // use owner ATAs for the fees, since there are no fees
          return [crateATA, ownerATA, ownerATA, ownerATA];
        });
      }
    })();
    const remainingAccounts = remainingAccountKeys.map(
      (acc): AccountMeta => ({
        pubkey: acc,
        isSigner: false,
        isWritable: true,
      })
    );

    const env = new TransactionEnvelope(this.provider, [
      ...additionalInstructions,
      this.programs.CrateRedeemInKind.instruction.redeem(amount.toU64(), {
        accounts: {
          withdrawAuthority: CRATE_REDEEM_IN_KIND_WITHDRAW_AUTHORITY,
          crateToken: crateKey,
          crateMint: amount.token.mintAccount,
          crateSource: ownerATAs.accounts.crate,
          owner,
          tokenProgram: TOKEN_PROGRAM_ID,
          crateTokenProgram: CRATE_ADDRESSES.CrateToken,
        },
        remainingAccounts,
      }),
    ]);
    env.instructions.unshift(...ownerATAs.instructions);
    return env;
  }
}
