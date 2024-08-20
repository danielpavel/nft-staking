use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{MasterEditionAccount, Metadata, MetadataAccount},
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::{user_account, StakeAccount, StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,
    pub collection: Account<'info, Mint>,

    pub config: Account<'info, StakeConfig>,

    // You add seed::program to force anchor to derive the metadta account from seeds and the
    // program id is overwritten with the metadata program id. The default is the current program id.
    #[account(
        seeds = [b"metadata", metadata_program.key().as_ref(), mint.key().as_ref()],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true,
    )]
    pub metadata: Account<'info, MetadataAccount>,

    // we make sure the master edition account for our mint exist
    #[account(
        seeds = [b"master_edition", metadata_program.key().as_ref(), mint.key().as_ref(), b"edition"],
        seeds::program = metadata_program.key(),
        bump
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,

    #[account(
        init,
        payer = user,
        seeds = [b"stake", mint.key().as_ref(), config.key().as_ref()],
        space = 8 + StakeAccount::INIT_SPACE,
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub rewards_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user
    )]
    pub rewards_ata: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> Claim<'info> {
    pub fn claim(&mut self) -> Result<()> {
        // mint a bunch of tokens.
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.rewards_mint.to_account_info(),
            to: self.rewards_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let amount =
            (self.user_account.points as u64) * 10_u64.pow(self.rewards_mint.decimals as u32);
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        mint_to(cpi_ctx, amount)?;

        self.user_account.points = 0;

        Ok(())
    }
}
