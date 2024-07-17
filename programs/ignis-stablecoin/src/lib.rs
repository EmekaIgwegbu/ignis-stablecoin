use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

declare_id!("Bou8TKf8G9iJQoZMptYtFHrpgvEjG4DTTXo8sxShLsht");

#[program]
pub mod ignis_stablecoin {
    use super::*;

    pub fn redeem_for_sister_coin(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    // TODO: I'm starting to think that this initialise function shouldn't be here since it's not a public-facing API. Or I need
    // to find a way to restrict access to the reserve authority, which would mean that the reserve authority would need to be predefined.
    // I think setting up the reserve wallet manually might make more sense. This way I can prevent a situation where a user passes in random token accounts that are supposedly
    // the ignis_reserve and ventura_reserve --> in the current implementation I think this would work as long as these accounts' mints match those of the user accounts.
    // Should the reserve wallet, mint account or token accounts somehow be derive from the program ID? Or maybe IgnisStablecoin should be derived from the program ID?

    pub fn initialise(ctx: Context<Initialise>, initial_supply: u64, decimals: u8) -> Result<()> {
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        // TODO: Invoke token program here to create ignis mint
        // ignis_stablecoin.mint =
        // ignis_stablecoin.name = "Ignis";
        // ignis_stablecoin.symbol = "IGS";
        ignis_stablecoin.peg = 1.0;
        ignis_stablecoin.reserve_authority = ctx.accounts.reserve_authority.key();

        // TODO: Invoke token program here to mint new ignis and move it to the newly created ignis reserve (ignis token account)
        // ignis_stablecoin.reserve =
        Ok(())
    }

    pub fn mint(ctx: Context<MintIgnis>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn burn(ctx: Context<BurnIgnis>, amount: u64) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct IgnisStablecoin {
    pub name: [u8; 32],
    pub symbol: [u8; 16],
    pub mint: Pubkey,    // The address of the mint account
    pub reserve: Pubkey, // The address of the token account that belongs to the ignis reserve
    pub peg: f64,
    pub reserve_authority: Pubkey,
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    // Change this to include space for other fields
    #[account(init, payer = reserve_authority, space = 8 + 32 + 16 + 32 + 8 + 32)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut)]
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, System>,
    pub system_program: Program<'info, System>,
    // TODO: does mint (account) need to be included here like
    // https://beta.solpg.io/https://github.com/solana-developers/anchor-examples/tree/main/account-constraints/token ?
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut, token::authority = user, token::mint = ignis_reserve.mint)]
    pub user_ignis_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, token::mint = ventura_reserve.mint)]
    pub user_ventura_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = reserve_authority)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(mut, token::authority = reserve_authority)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintIgnis<'info> {
    #[account(mut, token::authority = reserve_authority)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnIgnis<'info> {
    #[account(mut, token::authority = reserve_authority)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
