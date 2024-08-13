use anchor_lang::prelude::*;

declare_id!("HhEh32jLGdwZwWKUAZC753FCMQEh79S6vt7WmERtQhGc");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
