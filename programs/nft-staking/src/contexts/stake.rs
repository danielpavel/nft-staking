use anchor_lang::{prelude::*, solana_program::stake::instruction::StakeError};

use anchor_spl::{
    metadata::{
        mpl_token_metadata::instructions::{
            FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts, ThawDelegatedAccountCpi,
            ThawDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount, Metadata, MetadataAccount,
    },
    token::{approve, revoke, Approve, Mint, Revoke, Token, TokenAccount},
};

use crate::state::{StakeAccount, StakeConfig, UserAccount};

use crate::errors::StakeErrorCode;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,
    pub collection: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub mint_ata: Account<'info, TokenAccount>,

    // You add seed::program to force anchor to drive the metadta account from seeds and the
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
    pub config: Account<'info, StakeConfig>,

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
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> Stake<'info> {
    pub fn stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        // we do a cpi into the token program to delegate authority to the ATA
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Approve {
            to: self.mint_ata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        approve(cpi_ctx, 1)?;

        // once we have delegated authority to the stake account we can now freeze the token
        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let edition = &self.master_edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        FreezeDelegatedAccountCpi::new(
            metadata_program,
            FreezeDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        )
        .invoke()?;

        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),
            mint: self.mint.key(),
            last_update: Clock::get()?.unix_timestamp,
            bump: bumps.stake_account,
        });

        self.user_account.amount_staked += 1;

        Ok(())
    }

    pub fn unstake(&mut self, bumps: &StakeBumps) -> Result<()> {
        let elapsed = (Clock::get()?.unix_timestamp - self.stake_account.last_update) as u32;

        require!(
            elapsed > self.config.freeze_period,
            StakeErrorCode::UnstakeFreezePeriodNotElapsed
        );

        self.user_account.points +=
            (elapsed / (24 * 60 * 60)) * (self.config.points_per_stake as u32);

        // once we have delegated authority to the stake account we can now freeze the token
        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.mint_ata.to_account_info();
        let edition = &self.master_edition.to_account_info();
        let mint = &self.mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        ThawDelegatedAccountCpi::new(
            metadata_program,
            ThawDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        )
        .invoke()?;

        Ok(())
    }
}

//TODO implement claim and unstake
