import { expectTX } from "@saberhq/chai-solana";
import {
  PendingTransaction,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import {
  createMintAndVault,
  getMintInfo,
  getOrCreateATA,
  getOrCreateATAs,
  getTokenAccount,
  SPLToken,
  Token,
  TOKEN_PROGRAM_ID,
  TokenAmount,
  u64,
} from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";
import invariant from "tiny-invariant";

import type { CrateSDK } from "../src";
import { CRATE_FEE_OWNER } from "../src/constants";
import { makeSDK } from "./workspace";

describe("crate-token", () => {
  const sdk = makeSDK();
  const provider = sdk.provider;

  let crateToken: Token;

  let crateKey: PublicKey;
  let otherSDK: CrateSDK;
  let otherAccount: PublicKey;

  const makeUser = async (): Promise<{
    kp: Keypair;
    sdk: CrateSDK;
    tokenAccount: PublicKey;
  }> => {
    const kp = Keypair.generate();
    const newSDK = sdk.withSigner(kp);
    await expectTX(
      new PendingTransaction(
        newSDK.provider.connection,
        await newSDK.provider.connection.requestAirdrop(
          newSDK.provider.wallet.publicKey,
          LAMPORTS_PER_SOL
        )
      )
    ).to.be.fulfilled;

    const ata = await getOrCreateATA({
      provider: newSDK.provider,
      mint: crateToken.mintAccount,
      owner: kp.publicKey,
    });
    invariant(ata.instruction, "instruction");
    await expectTX(new TransactionEnvelope(newSDK.provider, [ata.instruction]))
      .to.be.fulfilled;
    return { kp, sdk: newSDK, tokenAccount: ata.address };
  };

  beforeEach(async () => {
    const mintKP = Keypair.generate();
    crateToken = Token.fromMint(mintKP.publicKey, 6);

    const { crateKey: theCrateKey, tx: createTX } = await sdk.newCrate({
      mintKP,
      decimals: crateToken.decimals,
    });
    crateKey = theCrateKey;
    await expectTX(createTX, "Create Crate Token").to.be.fulfilled;

    await expectTX(
      sdk.setWithdrawFee(theCrateKey, 1),
      "Set withdraw fee to 0.01%"
    ).to.be.fulfilled;

    const other = await makeUser();
    otherSDK = other.sdk;
    otherAccount = other.tokenAccount;
  });

  it("can issue and redeem", async () => {
    const amount = TokenAmount.parse(crateToken, "1000");
    await expectTX(
      await sdk.issue({
        amount,
        mintDestination: otherAccount,
      }),
      "Issue tokens to Other"
    ).to.be.fulfilled;

    await expectTX(
      await otherSDK.redeem({
        amount,
        underlyingTokens: [],
      }),
      "Redeem"
    ).to.be.fulfilled;
  });

  it("stakeholder fractions are correct", async () => {
    const userB = await makeUser();

    // create mints
    const [mintA, vaultA] = await createMintAndVault(
      provider,
      new u64("100000000")
    );
    const [mintB, vaultB] = await createMintAndVault(
      provider,
      new u64("100000000")
    );

    await expectTX(
      await sdk.issue({
        amount: TokenAmount.parse(crateToken, "500"),
        mintDestination: otherAccount,
      }),
      "Issue tokens to Other"
    ).to.be.fulfilled;
    await expectTX(
      await sdk.issue({
        amount: TokenAmount.parse(crateToken, "1000"),
        mintDestination: userB.tokenAccount,
      }),
      "Issue tokens to user2"
    ).to.be.fulfilled;

    expect(
      (await getTokenAccount(provider, otherAccount)).amount.toString()
    ).to.equal("500000000");
    expect(
      (await getTokenAccount(provider, userB.tokenAccount)).amount.toString()
    ).to.equal("1000000000");
    expect(
      (await getMintInfo(provider, crateToken.mintAccount)).supply.toString()
    ).to.equal("1500000000");

    // fees
    const { instructions: feeIXs } = await getOrCreateATAs({
      provider,
      mints: {
        mintA,
        mintB,
      },
      owner: CRATE_FEE_OWNER,
    });
    await expectTX(new TransactionEnvelope(provider, feeIXs.slice()), "fees").to
      .be.fulfilled;

    // send tokens to the crate
    const {
      instructions,
      accounts: { mintA: crateTokenA, mintB: crateTokenB },
    } = await getOrCreateATAs({
      provider,
      mints: {
        mintA,
        mintB,
      },
      owner: crateKey,
    });
    await expectTX(
      new TransactionEnvelope(provider, [
        ...instructions,
        SPLToken.createTransferInstruction(
          TOKEN_PROGRAM_ID,
          vaultA,
          crateTokenA,
          sdk.provider.wallet.publicKey,
          [],
          9_000000
        ),
        SPLToken.createTransferInstruction(
          TOKEN_PROGRAM_ID,
          vaultB,
          crateTokenB,
          sdk.provider.wallet.publicKey,
          [],
          18_000000
        ),
      ])
    ).to.be.fulfilled;

    const {
      accounts: { mintA: userAtokA, mintB: userAtokB },
      instructions: instructionsUserA,
    } = await getOrCreateATAs({
      provider,
      mints: {
        mintA,
        mintB,
      },
      owner: otherSDK.provider.wallet.publicKey,
    });
    const {
      accounts: { mintA: userBtokA, mintB: userBtokB },
      instructions: instructionsUserB,
    } = await getOrCreateATAs({
      provider,
      mints: {
        mintA,
        mintB,
      },
      owner: userB.sdk.provider.wallet.publicKey,
    });

    await expectTX(
      new TransactionEnvelope(provider, [
        ...instructionsUserA,
        ...instructionsUserB,
      ]),
      "ATAs"
    ).to.be.fulfilled;

    const underlyingTokens = [
      Token.fromMint(mintA, 6),
      Token.fromMint(mintB, 6),
    ];

    expect(
      (await getMintInfo(provider, crateToken.mintAccount)).supply.toString()
    ).to.equal("1500000000");

    await expectTX(
      await otherSDK.redeem({
        amount: TokenAmount.parse(crateToken, "500"),
        underlyingTokens,
      }),
      "Redeem A"
    ).to.be.fulfilled;

    const withFees = (amt: string): string =>
      new u64(amt).sub(new u64(amt).div(new u64("10000"))).toString();

    expect(
      (await getTokenAccount(provider, userAtokA)).amount.toString()
    ).to.equal(withFees("3000000"));
    expect(
      (await getTokenAccount(provider, userAtokB)).amount.toString()
    ).to.equal(withFees("6000000"));
    expect(
      (await getTokenAccount(provider, crateTokenA)).amount.toString()
    ).to.equal("6000000");
    expect(
      (await getTokenAccount(provider, crateTokenB)).amount.toString()
    ).to.equal("12000000");
    expect(
      (await getMintInfo(provider, crateToken.mintAccount)).supply.toString()
    ).to.equal("1000000000");

    expect(
      (await getTokenAccount(provider, otherAccount)).amount.toString()
    ).to.equal("0");
    expect(
      (await getTokenAccount(provider, userB.tokenAccount)).amount.toString()
    ).to.equal("1000000000");

    await expectTX(
      await userB.sdk.redeem({
        amount: TokenAmount.parse(crateToken, "1000"),
        underlyingTokens,
      }),
      "Redeem B"
    ).to.be.fulfilled;

    expect(
      (await getTokenAccount(provider, userBtokA)).amount.toString()
    ).to.equal(withFees("6000000"));
    expect(
      (await getTokenAccount(provider, userBtokB)).amount.toString()
    ).to.equal(withFees("12000000"));
    expect(
      (await getTokenAccount(provider, crateTokenA)).amount.toString()
    ).to.equal("0");
    expect(
      (await getTokenAccount(provider, crateTokenB)).amount.toString()
    ).to.equal("0");

    expect(
      (await getTokenAccount(provider, otherAccount)).amount.toString()
    ).to.equal("0");
    expect(
      (await getTokenAccount(provider, userB.tokenAccount)).amount.toString()
    ).to.equal("0");
    expect(
      (await getMintInfo(provider, crateToken.mintAccount)).supply.toString()
    ).to.equal("0");
  });
});
