//! Token-2022 mint checks that are independent of instruction context.
//!
//! Vesting amounts are recorded as exact token units. A TransferFeeConfig mint
//! would make the vault receive less than `total_amount` on create, and later
//! claims would try to pay out more than the vault holds.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions},
    state::Mint as Token2022Mint,
};

/// True when `mint` is a Token-2022 mint with the transfer-fee extension.
/// Fail closed on Token-2022 unpack errors so a malformed mint cannot skip the gate.
pub fn has_transfer_fee(mint: &AccountInfo) -> bool {
    let Ok(data) = mint.try_borrow_data() else {
        return mint.owner == &anchor_spl::token_2022::ID;
    };
    mint_data_has_transfer_fee(mint.owner, &data)
}

pub fn mint_data_has_transfer_fee(owner: &Pubkey, data: &[u8]) -> bool {
    if owner != &anchor_spl::token_2022::ID {
        return false;
    }
    let Ok(mint) = StateWithExtensions::<Token2022Mint>::unpack(data) else {
        return true;
    };
    mint.get_extension::<TransferFeeConfig>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_spl::token_2022::spl_token_2022::{
        extension::{
            transfer_fee::{TransferFee, TransferFeeConfig},
            BaseStateWithExtensionsMut, ExtensionType, PodStateWithExtensionsMut,
        },
        pod::{PodCOption, PodMint},
    };
    use solana_program_pack::Pack;
    use spl_token_interface::state::Mint;

    fn pack_classic_mint() -> Vec<u8> {
        let mint = Mint {
            mint_authority: solana_program_option::COption::None,
            supply: 1_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: solana_program_option::COption::None,
        };
        let mut data = vec![0u8; Mint::LEN];
        Pack::pack(mint, &mut data).unwrap();
        data
    }

    fn pack_fee_mint(fee_bps: u16) -> Vec<u8> {
        let mint_len = ExtensionType::try_calculate_account_len::<Token2022Mint>(&[
            ExtensionType::TransferFeeConfig,
        ])
        .unwrap();
        let mut data = vec![0u8; mint_len];
        let mut state =
            PodStateWithExtensionsMut::<PodMint>::unpack_uninitialized(&mut data).unwrap();
        let fee = TransferFee {
            epoch: 0.into(),
            maximum_fee: u64::MAX.into(),
            transfer_fee_basis_points: fee_bps.into(),
        };
        let ext = state.init_extension::<TransferFeeConfig>(true).unwrap();
        ext.older_transfer_fee = fee;
        ext.newer_transfer_fee = fee;
        *state.base = PodMint {
            mint_authority: PodCOption::none(),
            supply: 1_000.into(),
            decimals: 6,
            is_initialized: true.into(),
            freeze_authority: PodCOption::none(),
        };
        state.init_account_type().unwrap();
        data
    }

    #[test]
    fn classic_spl_mint_has_no_transfer_fee() {
        assert!(!mint_data_has_transfer_fee(
            &spl_token_interface::ID,
            &pack_classic_mint()
        ));
    }

    #[test]
    fn token_2022_mint_with_nonzero_fee_is_detected() {
        assert!(mint_data_has_transfer_fee(
            &anchor_spl::token_2022::ID,
            &pack_fee_mint(100)
        ));
    }

    #[test]
    fn token_2022_mint_without_fee_extension_is_allowed() {
        let mut data = pack_classic_mint();
        // Token-2022 mints with no extensions are still the base mint layout.
        assert!(!mint_data_has_transfer_fee(
            &anchor_spl::token_2022::ID,
            &data
        ));
        data.clear();
        assert!(mint_data_has_transfer_fee(
            &anchor_spl::token_2022::ID,
            &data
        ));
    }
}
