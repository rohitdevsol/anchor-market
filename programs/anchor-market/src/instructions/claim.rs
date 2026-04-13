use anchor_lang::prelude::*;
use anchor_spl::{
    token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked },
};
use crate::{ Market, error::PredictionMarketError, state::WinningOutcome };

#[derive(Accounts)]
#[instruction(market_id:u32)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id
    )]
    pub market: Box<Account<'info, Market>>,

    pub collateral_mint: InterfaceAccount<'info, Mint>, // USDC one

    #[account(
        mut,
        constraint = user_collateral.mint == market.collateral_mint,
        constraint = user_collateral.owner == user.key()
    )]
    pub user_collateral: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault
    )]
    pub collateral_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = outcome_a_mint.key() == market.outcome_a_mint
    )]
    pub outcome_a_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = outcome_b_mint.key() == market.outcome_b_mint
    )]
    pub outcome_b_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = user_outcome_a.mint == market.outcome_a_mint,
        constraint = user_outcome_a.owner == user.key()
    )]
    pub user_outcome_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_outcome_b.mint == market.outcome_b_mint,
        constraint = user_outcome_b.owner == user.key()
    )]
    pub user_outcome_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> ClaimRewards<'info> {
    pub fn claim_rewards(&mut self, market_id: u32) -> Result<()> {
        require!(self.market.is_settled, PredictionMarketError::MarketNotSettled);

        let winner = self.market.winning_outcome.ok_or_else(||
            error!(PredictionMarketError::InvalidWinningOutcome)
        )?;

        let (winner_mint_info, user_winner_ata) = match winner {
            WinningOutcome::OutcomeA =>
                (self.outcome_a_mint.to_account_info(), &self.user_outcome_a),
            _ => (self.outcome_b_mint.to_account_info(), &self.user_outcome_b),
        };

        let amount = user_winner_ata.amount;

        // burn tokens from the user ata

        token_interface::burn(
            CpiContext::new(self.token_program.key(), token_interface::Burn {
                mint: winner_mint_info,
                from: user_winner_ata.to_account_info(),
                authority: self.user.to_account_info(),
            }),
            amount
        )?;

        let binding = self.market.market_id.to_le_bytes();

        let seeds = &[b"market".as_ref(), binding.as_ref(), &[self.market.bump]];

        let signer = &[&seeds[..]];
        // transfer the usdc from collateral vault
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                TransferChecked {
                    from: self.collateral_vault.to_account_info(),
                    to: self.user_collateral.to_account_info(),
                    authority: self.market.to_account_info(),
                    mint: self.collateral_mint.to_account_info(),
                },
                signer
            ),
            amount,
            self.collateral_mint.decimals
        )?;

        self.market.total_collateral_locked = self.market.total_collateral_locked
            .checked_sub(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!("Claimed {} collateral for winning side", amount);

        Ok(())
    }
}
