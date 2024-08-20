use anchor_lang::prelude::*;

declare_id!("HhEh32jLGdwZwWKUAZC753FCMQEh79S6vt7WmERtQhGc");

mod contexts;
mod errors;
mod state;

pub use contexts::*;

#[program]
pub mod nft_staking {

    use super::*;

    pub fn init_config(
        ctx: Context<InitializeConfig>,
        points_per_stake: u8,
        max_stake: u8,
        freeze_period: u32,
    ) -> Result<()> {
        msg!("Initialize Config {:?}", ctx.program_id);

        ctx.accounts
            .initialize_config(points_per_stake, max_stake, freeze_period, &ctx.bumps)?;

        Ok(())
    }

    pub fn init_user(ctx: Context<InitializeUser>) -> Result<()> {
        msg!("Initialize User {:?}", ctx.program_id);

        ctx.accounts.initialize_user(&ctx.bumps)?;

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        msg!("Stake {:?}", ctx.program_id);
        ctx.accounts.stake(&ctx.bumps)?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        msg!("Unstake {:?}", ctx.program_id);
        ctx.accounts.unstake(&ctx.bumps)?;

        Ok(())
    }
}
