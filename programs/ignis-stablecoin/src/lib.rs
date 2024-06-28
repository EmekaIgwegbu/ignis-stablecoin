use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg8b4x6vKz2");

#[program]
pub mod algorithmic_stablecoin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, supply: u64) -> ProgramResult {
        let stablecoin = &mut ctx.accounts.stablecoin;
        stablecoin.supply = supply;
        stablecoin.price = 1.0; // Initial price
        Ok(())
    }

    pub fn mint(ctx: Context<MintStablecoin>, amount: u64) -> ProgramResult {
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info().clone(),
            to: ctx.accounts.token_account.to_account_info().clone(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;

        let stablecoin = &mut ctx.accounts.stablecoin;
        stablecoin.supply += amount;

        Ok(())
    }

    pub fn burn(ctx: Context<BurnStablecoin>, amount: u64) -> ProgramResult {
        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info().clone(),
            to: ctx.accounts.to.to_account_info().clone(),
            authority: ctx.accounts.authority.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let stablecoin = &mut ctx.accounts.stablecoin;
        stablecoin.supply -= amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8)]
    pub stablecoin: Account<'info, Stablecoin>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, Stablecoin>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, Stablecoin>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Stablecoin {
    pub supply: u64,
    pub price: f64,
}
