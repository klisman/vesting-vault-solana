#![allow(dead_code)]

use {
    litesvm::{types::TransactionResult, LiteSVM},
    solana_account::Account,
    solana_instruction::error::InstructionError,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    solana_transaction_error::TransactionError,
    spl_associated_token_account_interface::address::get_associated_token_address_with_program_id,
    spl_token_interface::state::{Account as TokenAccount, AccountState, Mint},
};

use anchor_lang::{
    prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize, InstructionData,
    ToAccountMetas,
};
use vesting_vault::{error::VestingError, CreateVestingParams};

pub const TOKEN_PROGRAM_ID: Pubkey = spl_token_interface::ID;
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = spl_associated_token_account_interface::program::ID;
pub const GRANT_ID: [u8; 32] = [7u8; 32];
pub const DECIMALS: u8 = 6;
pub const TOTAL: u64 = 1_000_000;
pub const START: i64 = 1_700_000_000;
pub const CLIFF: i64 = START + 100;
pub const END: i64 = START + 1_000;

pub fn program_bytes() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/vesting_vault.so");
    std::fs::read(&path).unwrap_or_else(|err| {
        panic!(
            "failed to read {}: {err}. Run `anchor build --ignore-keys` first.",
            path.display()
        )
    })
}

pub fn setup_svm() -> (LiteSVM, Pubkey) {
    let program_id = vesting_vault::id();
    let mut svm = LiteSVM::new();
    svm.add_program(program_id, &program_bytes()).unwrap();
    (svm, program_id)
}

pub fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    svm.set_sysvar(&solana_clock::Clock {
        unix_timestamp,
        ..solana_clock::Clock::default()
    });
}

pub fn vesting_pda(program_id: &Pubkey, creator: &Pubkey, id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[vesting_vault::VESTING_SEED, creator.as_ref(), id.as_ref()],
        program_id,
    )
}

pub fn ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, &TOKEN_PROGRAM_ID)
}

pub fn insert_mint(svm: &mut LiteSVM, mint: Pubkey, supply: u64) {
    let mint_state = Mint {
        mint_authority: COption::None,
        supply,
        decimals: DECIMALS,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint_state, &mut data).unwrap();
    svm.set_account(
        mint,
        Account {
            lamports: 1_000_000_000,
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn insert_token_account(
    svm: &mut LiteSVM,
    address: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
) {
    let state = TokenAccount {
        mint,
        owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let mut data = vec![0u8; TokenAccount::LEN];
    TokenAccount::pack(state, &mut data).unwrap();
    svm.set_account(
        address,
        Account {
            lamports: 1_000_000_000,
            data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn token_amount(svm: &LiteSVM, address: &Pubkey) -> u64 {
    let account = svm.get_account(address).expect("token account");
    TokenAccount::unpack(&account.data).unwrap().amount
}

#[allow(clippy::result_large_err)]
pub fn send_ix(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) -> TransactionResult {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    let result = svm.send_transaction(tx);
    svm.expire_blockhash();
    result
}

pub fn assert_anchor_error(res: TransactionResult, expected: VestingError) {
    let failed = match res {
        Err(e) => e,
        Ok(meta) => panic!("expected {expected:?}, transaction succeeded: {meta:?}"),
    };
    let expected_code = u32::from(expected);
    match failed.err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(
                code,
                expected_code,
                "expected {expected:?} ({expected_code}), got Custom({code}); logs:\n{}",
                failed.meta.logs.join("\n")
            );
        }
        other => panic!(
            "expected {expected:?} ({expected_code}), got {other:?}; logs:\n{}",
            failed.meta.logs.join("\n")
        ),
    }
}

pub fn default_params(revocable: bool) -> CreateVestingParams {
    CreateVestingParams {
        start_ts: START,
        cliff_ts: CLIFF,
        end_ts: END,
        total_amount: TOTAL,
        revocable,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_vesting_ix(
    creator: Pubkey,
    beneficiary: Pubkey,
    mint: Pubkey,
    vesting: Pubkey,
    vault: Pubkey,
    creator_ata: Pubkey,
    id: [u8; 32],
    params: CreateVestingParams,
) -> Instruction {
    Instruction::new_with_bytes(
        vesting_vault::id(),
        &vesting_vault::instruction::CreateVesting { id, params }.data(),
        vesting_vault::accounts::CreateVesting {
            creator,
            beneficiary,
            mint,
            vesting,
            vault,
            creator_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn claim_ix(
    beneficiary: Pubkey,
    vesting: Pubkey,
    mint: Pubkey,
    vault: Pubkey,
    beneficiary_ata: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        vesting_vault::id(),
        &vesting_vault::instruction::Claim {}.data(),
        vesting_vault::accounts::Claim {
            beneficiary,
            vesting,
            mint,
            vault,
            beneficiary_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

pub fn revoke_ix(
    creator: Pubkey,
    vesting: Pubkey,
    mint: Pubkey,
    vault: Pubkey,
    creator_ata: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        vesting_vault::id(),
        &vesting_vault::instruction::Revoke {}.data(),
        vesting_vault::accounts::Revoke {
            creator,
            vesting,
            mint,
            vault,
            creator_ata,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub fn close_ix(creator: Pubkey, vesting: Pubkey, mint: Pubkey, vault: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        vesting_vault::id(),
        &vesting_vault::instruction::Close {}.data(),
        vesting_vault::accounts::Close {
            creator,
            vesting,
            mint,
            vault,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}

pub struct FundedGrant {
    pub creator: Keypair,
    pub beneficiary: Keypair,
    pub mint: Pubkey,
    pub vesting: Pubkey,
    pub vault: Pubkey,
    pub creator_ata: Pubkey,
    pub beneficiary_ata: Pubkey,
}

pub fn funded_grant(svm: &mut LiteSVM, program_id: &Pubkey, revocable: bool) -> FundedGrant {
    let creator = Keypair::new();
    let beneficiary = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&beneficiary.pubkey(), 10_000_000_000).unwrap();

    let mint = Pubkey::new_unique();
    insert_mint(svm, mint, TOTAL);

    let (vesting, _) = vesting_pda(program_id, &creator.pubkey(), &GRANT_ID);
    let vault = ata(&vesting, &mint);
    let creator_ata = ata(&creator.pubkey(), &mint);
    let beneficiary_ata = ata(&beneficiary.pubkey(), &mint);
    insert_token_account(svm, creator_ata, mint, creator.pubkey(), TOTAL);

    set_clock(svm, START);
    let res = send_ix(
        svm,
        &creator,
        create_vesting_ix(
            creator.pubkey(),
            beneficiary.pubkey(),
            mint,
            vesting,
            vault,
            creator_ata,
            GRANT_ID,
            default_params(revocable),
        ),
    );
    assert!(res.is_ok(), "create_vesting failed: {res:?}");

    FundedGrant {
        creator,
        beneficiary,
        mint,
        vesting,
        vault,
        creator_ata,
        beneficiary_ata,
    }
}

pub fn load_vesting(svm: &LiteSVM, vesting: &Pubkey) -> vesting_vault::Vesting {
    let account = svm.get_account(vesting).expect("vesting PDA");
    let mut data: &[u8] = &account.data;
    vesting_vault::Vesting::try_deserialize(&mut data).unwrap()
}
