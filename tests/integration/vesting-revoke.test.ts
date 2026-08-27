import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import type { Surfnet } from '@solana/surfpool';
import type { Address, TransactionSigner } from '@solana/kit';

import {
  END,
  START,
  TOTAL,
  claim,
  createGrant,
  createMintAndFundCreator,
  expectNothingToClaim,
  generateFundedSigner,
  makeClient,
  revoke,
  startDeployedSurfnet,
  tokenBalance,
  warpToUnixSeconds,
  type KitClient,
} from './helpers';

describe('vesting revoke mid-stream', () => {
  let surfnet: Surfnet;
  let creator: TransactionSigner;
  let beneficiary: TransactionSigner;
  let creatorClient: KitClient;
  let beneficiaryClient: KitClient;
  let mint: Address;
  let vesting: Address;
  let vault: Address;

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

  it('returns unvested to creator and lets beneficiary claim the snapshot', async () => {
    // 25% vested: elapsed 250 / duration 1000.
    warpToUnixSeconds(surfnet, START + 250);
    const vestedAtRevoke = TOTAL / 4n;

    await revoke({
      client: creatorClient,
      creator,
      vesting,
      mint,
      vault,
    });

    // Creator recovered unvested (75%).
    expect(await tokenBalance(creatorClient, creator.address, mint)).toBe(
      TOTAL - vestedAtRevoke,
    );

    await claim({
      client: beneficiaryClient,
      beneficiary,
      vesting,
      mint,
      vault,
    });
    expect(await tokenBalance(beneficiaryClient, beneficiary.address, mint)).toBe(
      vestedAtRevoke,
    );

    // After snapshot is claimed, further claims fail even past end.
    warpToUnixSeconds(surfnet, END + 10);
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
});
