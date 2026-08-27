import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  address,
  createClient,
  createKeyPairSignerFromBytes,
  generateKeyPairSigner,
  type Address,
  type TransactionSigner,
} from '@solana/kit';
import { solanaRpc } from '@solana/kit-plugin-rpc';
import { signer } from '@solana/kit-plugin-signer';
import {
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getCreateMintInstructionPlan,
  getMintToCheckedInstruction,
  TOKEN_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
} from '@solana-program/token';
import { Surfnet } from '@solana/surfpool';
import {
  getCreateVestingInstructionAsync,
  getClaimInstructionAsync,
  getRevokeInstructionAsync,
  findVestingPda,
  VESTING_VAULT_PROGRAM_ADDRESS,
  VESTING_VAULT_ERROR__NOTHING_TO_CLAIM,
} from '@vesting-vault/client';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

export const PROGRAM_ID = VESTING_VAULT_PROGRAM_ADDRESS;
export const DECIMALS = 6;
export const TOTAL = 1_000_000n;
export const GRANT_ID = new Uint8Array(32).fill(7);

/** Schedule in unix seconds (matches on-chain Clock). Leave room for Surfnet startup. */
export const START = Math.floor(Date.now() / 1000) + 60;
export const CLIFF = START + 100;
export const END = START + 1_000;

export type KitClient = Awaited<ReturnType<typeof makeClient>>;

export function soPath(): string {
  return path.join(repoRoot, 'target/deploy/vesting_vault.so');
}

export function idlPath(): string {
  return path.join(repoRoot, 'target/idl/vesting_vault.json');
}

export function startDeployedSurfnet(): Surfnet {
  const surfnet = Surfnet.start();
  surfnet.deploy({
    programId: PROGRAM_ID,
    soPath: soPath(),
    idlPath: idlPath(),
  });
  return surfnet;
}

/** Surfpool's typed helper takes milliseconds; Clock uses seconds. */
export function warpToUnixSeconds(surfnet: Surfnet, unixSeconds: number): void {
  surfnet.timeTravelToTimestamp(unixSeconds * 1000);
}

export async function makeClient(
  rpcUrl: string,
  wsUrl: string,
  keypair: TransactionSigner,
){
  return await createClient()
    .use(signer(keypair))
    .use(
      solanaRpc({
        rpcUrl,
        rpcSubscriptionsUrl: wsUrl,
      }),
    );
}

export async function generateFundedSigner(surfnet: Surfnet): Promise<TransactionSigner> {
  const s = await generateKeyPairSigner();
  surfnet.fundSol(s.address, 10_000_000_000);
  return s;
}

export async function signerFromSurfnetKeypair(
  info: ReturnType<typeof Surfnet.newKeypair>,
): Promise<TransactionSigner> {
  return createKeyPairSignerFromBytes(Uint8Array.from(info.secretKey));
}

export async function createMintAndFundCreator(
  client: KitClient,
  creator: TransactionSigner,
  amount: bigint = TOTAL,
): Promise<{ mint: Address; creatorAta: Address }> {
  const mintSigner = await generateKeyPairSigner();
  const mint = mintSigner.address;

  await client.sendTransaction(
    await getCreateMintInstructionPlan(client, {
      payer: creator,
      newMint: mintSigner,
      decimals: DECIMALS,
      mintAuthority: creator.address,
    }),
  );

  const [creatorAta] = await findAssociatedTokenPda({
    mint,
    owner: creator.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  await client.sendTransaction([
    await getCreateAssociatedTokenIdempotentInstructionAsync({
      payer: creator,
      owner: creator.address,
      mint,
    }),
    getMintToCheckedInstruction({
      mint,
      token: creatorAta,
      mintAuthority: creator,
      amount,
      decimals: DECIMALS,
    }),
  ]);

  return { mint, creatorAta };
}

export async function createGrant(opts: {
  client: KitClient;
  creator: TransactionSigner;
  beneficiary: Address;
  mint: Address;
  revocable?: boolean;
}): Promise<{ vesting: Address; vault: Address }> {
  const { client, creator, beneficiary, mint, revocable = true } = opts;
  const [vesting] = await findVestingPda({
    creator: creator.address,
    id: GRANT_ID,
  });
  const [vault] = await findAssociatedTokenPda({
    mint,
    owner: vesting,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const ix = await getCreateVestingInstructionAsync({
    creator,
    beneficiary,
    mint,
    vesting,
    vault,
    id: GRANT_ID,
    startTs: BigInt(START),
    cliffTs: BigInt(CLIFF),
    endTs: BigInt(END),
    totalAmount: TOTAL,
    revocable,
  });
  await client.sendTransaction([ix]);
  return { vesting, vault };
}

export async function claim(opts: {
  client: KitClient;
  beneficiary: TransactionSigner;
  vesting: Address;
  mint: Address;
  vault: Address;
}): Promise<void> {
  const { client, beneficiary, vesting, mint, vault } = opts;
  const [beneficiaryAta] = await findAssociatedTokenPda({
    mint,
    owner: beneficiary.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });
  const ix = await getClaimInstructionAsync({
    beneficiary,
    vesting,
    mint,
    vault,
    beneficiaryAta,
  });
  await client.sendTransaction([ix]);
}

export async function revoke(opts: {
  client: KitClient;
  creator: TransactionSigner;
  vesting: Address;
  mint: Address;
  vault: Address;
}): Promise<void> {
  const { client, creator, vesting, mint, vault } = opts;
  const [creatorAta] = await findAssociatedTokenPda({
    mint,
    owner: creator.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });
  const ix = await getRevokeInstructionAsync({
    creator,
    vesting,
    mint,
    vault,
    creatorAta,
  });
  await client.sendTransaction([ix]);
}

export async function tokenBalance(
  client: KitClient,
  owner: Address,
  mint: Address,
): Promise<bigint> {
  const [ata] = await findAssociatedTokenPda({
    mint,
    owner,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });
  return tokenAccountBalance(client, ata);
}

export async function tokenAccountBalance(
  client: KitClient,
  tokenAccount: Address,
): Promise<bigint> {
  const { value } = await client.rpc.getTokenAccountBalance(tokenAccount).send();
  return BigInt(value.amount);
}

export function expectNothingToClaim(err: unknown): void {
  const seen = new Set<object>();
  const describe = (value: unknown): string => {
    if (value === null || value === undefined) return String(value);
    if (typeof value !== 'object') return String(value);
    if (seen.has(value)) return '';
    seen.add(value);
    return Object.getOwnPropertyNames(value)
      .map((key) => `${key}:${describe((value as Record<string, unknown>)[key])}`)
      .join('\n');
  };
  const haystack = describe(err);
  if (
    haystack.includes('NothingToClaim') ||
    haystack.includes(String(VESTING_VAULT_ERROR__NOTHING_TO_CLAIM)) ||
    haystack.includes('6003') ||
    haystack.includes('0x1773')
  ) {
    return;
  }
  throw err instanceof Error ? err : new Error(haystack);
}

export { address, TOKEN_PROGRAM_ADDRESS };
