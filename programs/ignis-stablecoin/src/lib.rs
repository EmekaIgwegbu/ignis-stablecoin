use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::token::{self, InitializeMint, Mint, MintTo, Token, TokenAccount, Transfer};
use std::str::FromStr;

declare_id!("Bou8TKf8G9iJQoZMptYtFHrpgvEjG4DTTXo8sxShLsht");

#[program]
pub mod ignis_stablecoin {
    use super::*;

    pub fn initialise(ctx: Context<Initialise>, initial_supply: u64) -> Result<()> {
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        // TODO: Invoke token program here to create ignis mint

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = InitializeMint {
            mint: ctx.accounts.ignis_mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        // hardcode decimals to be 6 for both ignis and ventura mints

        // ignis_stablecoin.mint =
        // ignis_stablecoin.name = "Ignis";
        // ignis_stablecoin.symbol = "IGS";
        ignis_stablecoin.peg = 1.0;
        ignis_stablecoin.reserve_authority = ctx.accounts.reserve_authority.key();

        // Ensure that reserve accounts are created with the correct program derived addresses

        // TODO: Invoke token program here to mint new ignis and move it to the newly created ignis reserve (ignis token account)
        // ignis_stablecoin.reserve =
        Ok(())
    }

    pub fn redeem_ignis(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    pub fn redeem_ventura(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    pub fn mint_ignis(ctx: Context<MintIgnis>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn mint_ventura(ctx: Context<MintVentura>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn burn_ignis(ctx: Context<BurnIgnis>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn burn_ventura(ctx: Context<BurnVentura>, amount: u64) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct IgnisStablecoin {
    pub name: [u8; 32],
    pub symbol: [u8; 16],
    pub reserve_amount: u64, // The amount of ignis in reserves, as measured in microignis (millionths of ignis)
    pub circulating_supply: u64, // The amount of ignis in circulation (excludes reserves) as measured in microignis
    pub mint: Pubkey,            // The address of the mint account
    pub ignis_reserve: Pubkey, // The address of the token account that belongs to the ignis reserve
    pub peg: f64,
    pub reserve_authority: Pubkey,
}

#[account]
pub struct VenturaCoin {
    pub name: [u8; 32],
    pub symbol: [u8; 16],
    pub reserve_amount: u64, // The amount of ventura in reserves, as measured in microventura (millionths of ventura)
    pub circulating_supply: u64, // The amount of ventura in circulation (excludes reserves) as measured in microventura
    pub mint: Pubkey,            // The address of the mint account
    pub ventura_reserve: Pubkey, // The address of the token account that belongs to the ventura reserve
    pub reserve_authority: Pubkey,
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    // Change this to include space for other fields
    #[account(init, payer = reserve_authority, space = 8 + 32 + 16 + 32 + 32 + 8 + 32, seeds=[b"ignis-stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(init, payer = reserve_authority, space = Mint::LEN, seeds=[b"ignis-mint"], bump)]
    pub ignis_mint: Account<'info, Mint>,
    #[account(init, payer = reserve_authority, space = Mint::LEN, seeds=[b"ventura-mint"], bump)]
    pub ventura_mint: Account<'info, Mint>,
    // Create the ignis reserve account with a PDA that will be used to give this program authority over it
    #[account(init, payer = reserve_authority, space = TokenAccount::LEN, seeds=[b"ignis-reserve"], bump)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    // Create the ventura reserve account with a PDA that will be used to give this program authority over it
    #[account(init, payer = reserve_authority, space = TokenAccount::LEN, seeds=[b"ventura-reserve"], bump)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    // The address constraint ensures that only the predefined reserve wallet can authorise this instruction
    #[account(mut, address = Pubkey::from_str("52Ygg62kTvXgurKkyezpToHGvmU51CJxLXoEoZ25HnMm").unwrap())]
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, sysvar::rent::Rent>,
    // TODO: does mint (account) need to be included here like
    // https://beta.solpg.io/https://github.com/solana-developers/anchor-examples/tree/main/account-constraints/token ?
}

impl<'info> Initialise<'info> {
    // pub fn
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut, token::authority = user, token::mint = ignis_reserve.mint)]
    pub user_ignis_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, token::mint = ventura_reserve.mint)]
    pub user_ventura_account: Account<'info, TokenAccount>,
    // Constraint checks that the authority of this account is the PDA
    #[account(mut, token::authority = ignis_reserve, seeds = [b"ignis-reserve"], bump)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    // Constraint checks that the authority of this account is the PDA
    #[account(mut, token::authority = ventura_reserve, seeds = [b"ventura-reserve"], bump)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintIgnis<'info> {
    #[account(mut, token::authority = ignis_reserve, seeds = [b"ignis-reserve"], bump)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnIgnis<'info> {
    #[account(mut, token::authority = ignis_reserve, seeds = [b"ignis-reserve"], bump)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// TODO
#[derive(Accounts)]
pub struct MintVentura<'info> {}

// TODO
#[derive(Accounts)]
pub struct BurnVentura<'info> {}
