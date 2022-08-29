use anchor_lang::{
    prelude::*,
    solana_program::{
        program::invoke, program_memory::sol_memcmp, pubkey::PUBKEY_BYTES, system_instruction,
    },
};
use std::slice::Iter;

use metaplex_token_metadata::state::Metadata;
use solana_program::account_info::next_account_info;

declare_id!("royU9jdMmk47uMuFheod9o5dFijxUHtKsBuyTfrELZu");

pub fn is_metadata_for_mint(mint: &AccountInfo, metadata: &AccountInfo) -> bool {
    let (key, _bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.key().as_ref(),
        ],
        &mpl_token_metadata::id(),
    );
    if key != *metadata.key {
        return false;
    }
    true
}
pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> Result<()> {
    if sol_memcmp(key1.as_ref(), key2.as_ref(), PUBKEY_BYTES) != 0 {
        err!(Errors::PublicKeyMismatch)
    } else {
        Ok(())
    }
}


#[program]
pub mod creator_payer {

    use super::*;

    pub fn pay_creator<'info>(
        ctx: Context<'_, '_, '_, 'info, PayCreator<'info>>,
        fee: u64,
    ) -> Result<()> {
        require!(
            is_metadata_for_mint(
                &ctx.accounts.mint.to_account_info(),
                &ctx.accounts.metadata.to_account_info()
            ),
            Errors::MismatchedMetadataAddr
        );

        let metadata = Metadata::from_account_info(&ctx.accounts.metadata.to_account_info())?;
        let total_fee = fee;
        let mut remaining_fee = fee;
        let remaining_accounts_iter: &mut Iter<AccountInfo<'info>> =
            &mut ctx.remaining_accounts.iter();
        let payer = ctx.accounts.payer.to_account_info();
        let system_program = ctx.accounts.system_program.to_account_info();
        match metadata.data.creators {
            Some(creators) => {
                for creator in creators {
                    let pct = creator.share as u128;
                    let creator_fee =
                        pct.checked_mul(total_fee as u128)
                            .ok_or(Errors::NumericalOverflow)?
                            .checked_div(100)
                            .ok_or(Errors::NumericalOverflow)? as u64;
                    remaining_fee = remaining_fee
                        .checked_sub(creator_fee)
                        .ok_or(Errors::NumericalOverflow)?;
                    let current_creator_info = next_account_info(remaining_accounts_iter)?;

                    assert_keys_equal(creator.address, *current_creator_info.key)?;
                    if creator_fee > 0 {
                        let instruction = system_instruction::transfer(
                            payer.key,
                            current_creator_info.key,
                            creator_fee,
                        );
                        invoke(
                            &instruction,
                            &[
                                payer.clone(),
                                current_creator_info.clone(),
                                system_program.clone(),
                            ],
                        )?;
                    }
                }
            }
            None => {
                msg!("No creators found in metadata");
            }
        };

        Ok(())
    }
}

#[error_code]
pub enum Errors {
    #[msg("Metadata account is not for the mint specified")]
    MismatchedMetadataAddr,
    #[msg("NumericalOverflow")]
    NumericalOverflow,
    PublicKeyMismatch,
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(fee: u64)]
pub struct PayCreator<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: ignore
    pub mint: UncheckedAccount<'info>,
    /// CHECK: ignore
    pub metadata: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}
