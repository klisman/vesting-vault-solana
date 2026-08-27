import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import type { Surfnet } from '@solana/surfpool';
import type { TransactionSigner } from '@solana/kit';

import {
  CLIFF,
  END,
  START,
  TOTAL,
  claim,
  createGrant,
  createMintAndFundCreator,
  expectNothingToClaim,
  generateFundedSigner,
  makeClient,
  startDeployedSurfnet,
  tokenBalance,
  warpToUnixSeconds,
  type KitClient,
} from './helpers';

describe('vesting time travel', () => {
  let surfnet: Surfnet;
  let creator: TransactionSigner;
  let beneficiary: TransactionSigner;
  let creatorClient: KitClient;
  let beneficiaryClient: KitClient;
  let mint: Awaited<ReturnType<typeof createMintAndFundCreator>>['mint'];
  let vesting: Awaited<ReturnType<typeof createGrant>>['vesting'];
  let vault: Awaited<ReturnType<typeof createGrant>>['vault'];

  beforeAll(async () => {
    surfnet = startDeployedSurfnet();
    creator = await generateFundedSigner(surfnet);
    beneficiary = await generateFundedSigner(surfnet);
    creatorClient = await makeClient(surfnet.rpcUrl, surfnet.wsUrl, creator);
    beneficiaryClient = await makeClient(surfnet.rpcUrl, surfnet.wsUrl, beneficiary);

    warpToUnixSeconds(surfnet, START);
    ({ mint } = await createMintAndFundCreator(creatorClient, creator));
    ({ vesting, vault } = await createGrant({
      client: creatorClient,
      creator,
      beneficiary: beneficiary.address,
      mint,
      revocable: true,
    }));
  });

  afterAll(() => {
    surfnet?.stop();
  });

  it('rejects claim before cliff with NothingToClaim', async () => {
    warpToUnixSeconds(surfnet, CLIFF - 1);
    await expect(
      claim({
        client: beneficiaryClient,
        beneficiary,
        vesting,
        mint,
        vault,
      }),
    ).rejects.toSatisfy((err: unknown) => {
      expectNothingToClaim(err);
      return true;
    });
  });

  it('claims linearly after cliff and fully at end', async () => {
    // Midpoint of start→end: 500/1000 of TOTAL.
    warpToUnixSeconds(surfnet, START + 500);
    await claim({
      client: beneficiaryClient,
      beneficiary,
      vesting,
      mint,
      vault,
    });
    expect(await tokenBalance(beneficiaryClient, beneficiary.address, mint)).toBe(TOTAL / 2n);

    warpToUnixSeconds(surfnet, END);
    await claim({
      client: beneficiaryClient,
      beneficiary,
      vesting,
      mint,
      vault,
    });
    expect(await tokenBalance(beneficiaryClient, beneficiary.address, mint)).toBe(TOTAL);
  });
});
