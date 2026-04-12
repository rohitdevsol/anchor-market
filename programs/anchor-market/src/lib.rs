use anchor_lang::prelude::*;

mod constants;
mod error;
mod instructions;
mod state;

use constants::*;
use instructions::*;
use state::*;

declare_id!("7JAt2fYYm4L9ZYTeZiN3j7eQtUGwCHiCjk9gn3MEfx55");

#[program]
pub mod anchor_market {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeMarket>,
        market_id: u32,
        expiry_ts: i64
    ) -> Result<()> {
        ctx.accounts.initilize_market(market_id, expiry_ts, ctx.bumps)?;
        Ok(())
    }

    pub fn split_tokens(ctx: Context<SplitToken>, market_id: u32, amount: u64) -> Result<()> {
        ctx.accounts.split_token(market_id, amount)?;
        Ok(())
    }

    pub fn merge_tokens(ctx: Context<MergeToken>, market_id: u32, amount: u64) -> Result<()> {
        ctx.accounts.merge_tokens(market_id, amount)?;
        Ok(())
    }
}
