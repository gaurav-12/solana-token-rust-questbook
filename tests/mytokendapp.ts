import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import * as common from "@project-serum/common";
import { TokenInstructions } from "@project-serum/serum";
import { assert } from "chai";
import { Mytokendapp } from "../target/types/mytokendapp";

describe("mytokendapp", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Mytokendapp as Program<Mytokendapp>;

  let mint = null;
  let from = null;
  let to = null;

  it("Initializes test state", async () => {
    mint = await createMint(provider);
    from = await createTokenAccount(provider, mint, provider.wallet.publicKey);
    to = await createTokenAccount(provider, mint, provider.wallet.publicKey);
  });

  it("Mints a token", async () => {
    const mintAmount = new anchor.BN(1000);

    // Function name converted from snake_case(Rust/Program) to camelCase(JS)
    await program.rpc.proxyMintTo(mintAmount, {
      accounts: {
        authority: provider.wallet.publicKey,
        mint,
        to: from,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });

    const fromAccount = await getTokenAccount(provider, from);
    assert.ok(fromAccount.amount.eq(mintAmount));
  });

  it("Transfers a token", async () => {
    await program.rpc.proxyTransfer(new anchor.BN(400), {
      accounts: {
        authority: provider.wallet.publicKey,
        from: from,
        to: to,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });

    const fromAccount = await getTokenAccount(provider, from);
    const toAccount = await getTokenAccount(provider, to);

    // Minted 1000 in last testcase, transferred 400
    assert.ok(fromAccount.amount.eq(new anchor.BN(600)));
    assert.ok(toAccount.amount.eq(new anchor.BN(400)));
  });

  it("Burns a token", async () => {
    // Burn 300 from 'to' account out of received 400 in previous testcase
    await program.rpc.proxyBurn(new anchor.BN(300), {
      accounts: {
        authority: provider.wallet.publicKey,
        mint,
        to,
        tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
      },
    });

    const toAccount = await getTokenAccount(provider, to);
    assert.ok(toAccount.amount.eq(new anchor.BN(100)));
  });

  it("Set new mint authority", async () => {
    const newMintAuthority = anchor.web3.Keypair.generate();
    await program.rpc.proxySetAuthority(
      { mintTokens: {} },
      newMintAuthority.publicKey,
      {
        accounts: {
          accountOrMint: mint,
          currentAuthority: provider.wallet.publicKey,
          tokenProgram: TokenInstructions.TOKEN_PROGRAM_ID,
        },
      }
    );

    const mintInfo = await getMintInfo(provider, mint);
    assert.ok(mintInfo.mintAuthority.equals(newMintAuthority.publicKey));
  });
});

const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  TokenInstructions.TOKEN_PROGRAM_ID.toString()
);

async function getTokenAccount(provider: anchor.Provider, addr) {
  return await common.getTokenAccount(provider, addr);
}

async function getMintInfo(provider: anchor.Provider, mintAddr) {
  return await common.getMintInfo(provider, mintAddr);
}

async function createMint(provider: anchor.Provider, authority?) {
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = anchor.web3.Keypair.generate();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey
  );

  const tx = new anchor.web3.Transaction();
  tx.add(...instructions);

  await provider.send(tx, [mint]);

  return mint.publicKey;
}

async function createMintInstructions(
  provider: anchor.Provider,
  authority,
  mint
) {
  let instructions = [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: mint,
      space: 82,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeMint({
      mint,
      decimals: 0,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}

async function createTokenAccount(provider: anchor.Provider, mint, owner) {
  const vault = anchor.web3.Keypair.generate();
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createTokenAccountInstructions(
      provider,
      vault.publicKey,
      mint,
      owner
    ))
  );
  await provider.send(tx, [vault]);
  return vault.publicKey;
}

async function createTokenAccountInstructions(
  provider: anchor.Provider,
  newAccountPubkey,
  mint,
  owner,
  lamports?
) {
  if (lamports === undefined) {
    lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
  }
  return [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey,
      space: 165,
      lamports,
      programId: TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: newAccountPubkey,
      mint,
      owner,
    }),
  ];
}
