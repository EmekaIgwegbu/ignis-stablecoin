use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Burn, InitializeAccount, InitializeMint, Mint, MintTo, Token, TokenAccount, Transfer,
};
use std::str::FromStr;

declare_id!("Bou8TKf8G9iJQoZMptYtFHrpgvEjG4DTTXo8sxShLsht");

#[program]
pub mod ignis_stablecoin {
    use super::*;

    pub fn initialise(
        ctx: Context<Initialise>,
        initial_ignis_supply: u64,
        initial_ventura_supply: u64,
    ) -> Result<()> {
        let bumps = ctx.bumps;
        // Initialise ignis mint
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = InitializeMint {
            mint: ctx.accounts.ignis_mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let signer_seeds = &[b"ignis-mint", &[bumps.ignis_mint][..]][..];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[signer_seeds]);
        token::initialize_mint(
            cpi_context,
            6,                                             // ignis decimal precision
            ctx.accounts.ignis_mint.to_account_info().key, // set the mint_authority to the ignis_mint PDA
            Some(ctx.accounts.ignis_mint.to_account_info().key),
        );

        // Initialize ventura mint
        let cpi_accounts = InitializeMint {
            mint: ctx.accounts.ventura_mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let signer_seeds = &[b"ventura-mint", &[bumps.ventura_mint][..]][..];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[signer_seeds]);
        token::initialize_mint(
            cpi_context,
            6,                                               // ventura decimal precision
            ctx.accounts.ventura_mint.to_account_info().key, // set the mint_authority to the ventura_mint PDA
            Some(ctx.accounts.ventura_mint.to_account_info().key),
        );

        // Initialize ignis reserve
        let cpi_accounts = InitializeAccount {
            account: ctx.accounts.ignis_reserve.to_account_info(),
            mint: ctx.accounts.ignis_mint.to_account_info(),
            authority: ctx.accounts.ignis_reserve.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let signer_seeds = &[b"ignis-reserve", &[bumps.ignis_reserve][..]][..];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[signer_seeds]);
        token::initialize_account(cpi_context);

        // Initialize ventura reserve
        let cpi_accounts = InitializeAccount {
            account: ctx.accounts.ventura_reserve.to_account_info(),
            mint: ctx.accounts.ventura_mint.to_account_info(),
            authority: ctx.accounts.ventura_reserve.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let signer_seeds = &[b"ventura-reserve", &[bumps.ventura_reserve][..]][..];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &[signer_seeds]);
        token::initialize_account(cpi_context);

        // Mint ignis to the reserve
        let cpi_accounts = MintTo {
            authority: ctx.accounts.ignis_reserve.to_account_info(),
            mint: ctx.accounts.ignis_mint.to_account_info(),
            to: ctx.accounts.ignis_reserve.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_context, initial_ignis_supply);

        // Mint ventura to the reserve
        let cpi_accounts = MintTo {
            authority: ctx.accounts.ventura_reserve.to_account_info(),
            mint: ctx.accounts.ventura_mint.to_account_info(),
            to: ctx.accounts.ventura_reserve.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_context, initial_ventura_supply);

        // Initialize ignis stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        // TODO: Either figure out a way to serialize strings into a fixed length byte array or link this account to token metadata
        ignis_stablecoin.reserve_amount = initial_ignis_supply;
        ignis_stablecoin.circulating_supply = 0;
        ignis_stablecoin.mint = ctx.accounts.ignis_mint.to_account_info().key();
        ignis_stablecoin.ignis_reserve = ctx.accounts.ignis_reserve.to_account_info().key();
        ignis_stablecoin.peg = 1.0;
        ignis_stablecoin.reserve_authority = ctx.accounts.reserve_authority.key();

        // Initialize ventura coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.reserve_amount = initial_ventura_supply;
        ventura_coin.circulating_supply = 0;
        ventura_coin.mint = ctx.accounts.ventura_mint.to_account_info().key();
        ventura_coin.ventura_reserve = ctx.accounts.ventura_reserve.to_account_info().key();
        ventura_coin.reserve_authority = ctx.accounts.reserve_authority.key();

        Ok(())
    }

    pub fn redeem_ignis(ctx: Context<Redeem>, amount: u64) -> Result<()> {
        // TODO: Ensure that ventura is listed on the market and fetch the latest market data

        // Burn ignis from the user's account
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Burn {
            mint: ctx.accounts.ignis_mint.to_account_info(),
            from: ctx.accounts.user_ignis_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        let cpi = token::burn(cpi_context, amount);

        // Update ignis stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.circulating_supply -= amount;

        // TODO: Calculate the equivalent ventura_amount here using the market data

        // Mint ventura to the user's account
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = MintTo {
            authority: ctx.accounts.user.to_account_info(),
            mint: ctx.accounts.ventura_mint.to_account_info(),
            to: ctx.accounts.user_ventura_account.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        let cpi = token::mint_to(cpi_context, ventura_amount);

        // Update ventura coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.circulating_supply += ventura_amount;
        Ok(())
    }

    pub fn redeem_ventura(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    pub fn mint_ignis(ctx: Context<MintIgnis>, amount: u64) -> Result<()> {
        // Mint ignis to the reserve
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = MintTo {
            authority: ctx.accounts.ignis_reserve.to_account_info(),
            mint: ctx.accounts.ignis_mint.to_account_info(),
            to: ctx.accounts.ignis_reserve.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_context, amount);

        // Update stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.reserve_amount += amount;
        Ok(())
    }

    pub fn mint_ventura(ctx: Context<MintVentura>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn burn_ignis(ctx: Context<BurnIgnis>, amount: u64) -> Result<()> {
        // Burn ignis from the reserve
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Burn {
            mint: ctx.accounts.ignis_mint.to_account_info(),
            from: ctx.accounts.ignis_reserve.to_account_info(),
            authority: ctx.accounts.ignis_reserve.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_context, amount);

        // Update stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.reserve_amount -= amount;
        Ok(())
    }

    pub fn burn_ventura(ctx: Context<BurnVentura>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn fetch_ignis_market_data(ctx: Context<FetchIgnisMarketData>) -> Result<()> {
        Ok(())
    }

    pub fn fetch_ventura_market_data(ctx: Context<FetchVenturaMarketData>) -> Result<()> {
        Ok(())
    }
}

#[account]
pub struct IgnisStablecoin {
    // TODO: Include name, symbol, image and other metadata
    pub reserve_amount: u64, // The amount of ignis in reserves, as measured in microignis (millionths of ignis)
    pub circulating_supply: u64, // The amount of ignis in circulation (excludes reserves) as measured in microignis
    pub mint: Pubkey,            // The address of the mint account
    pub ignis_reserve: Pubkey, // The address of the token account that belongs to the ignis reserve
    pub peg: f64,
    pub reserve_authority: Pubkey, // The party that has authority over the coin reserves.
}

#[account]
pub struct VenturaCoin {
    // TODO: Include name, symbol, image and other metadata
    pub reserve_amount: u64, // The amount of ventura in reserves, as measured in microventura (millionths of ventura)
    pub circulating_supply: u64, // The amount of ventura in circulation (excludes reserves) as measured in microventura
    pub mint: Pubkey,            // The address of the mint account
    pub ventura_reserve: Pubkey, // The address of the token account that belongs to the ventura reserve
    pub reserve_authority: Pubkey, // The party that has authority over the coin reserves.
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    // Change this to include space for other fields
    #[account(init, payer = reserve_authority, space = 8 + 32 + 16 + 8 + 8 + 32 + 32 + 8 + 32, seeds=[b"ignis-stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(init, payer = reserve_authority, space = 8 + 32 + 16 + 8 + 8 + 32 + 32 + 32, seeds=[b"ventura-coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
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
    pub rent: Sysvar<'info, Rent>,
}

// impl<'info> Initialise<'info> {
//     // pub fn
// }

#[derive(Accounts)]
pub struct Redeem<'info> {
    #[account(mut, seeds = [b"ignis-stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, seeds = [b"ventura-coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(mut, token::authority = user, token::mint = ignis_stablecoin.mint)]
    pub user_ignis_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, token::mint = ventura_coin.mint)]
    pub user_ventura_account: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    #[account(mut, address = ventura_coin.mint)]
    pub ventura_mint: Account<'info, Mint>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintIgnis<'info> {
    #[account(mut, has_one = reserve_authority, seeds = [b"ignis-stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, address = ignis_stablecoin.ignis_reserve)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnIgnis<'info> {
    #[account(mut, has_one = reserve_authority, seeds = [b"ignis-stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, address = ignis_stablecoin.ignis_reserve)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// TODO
#[derive(Accounts)]
pub struct MintVentura {}

// TODO
#[derive(Accounts)]
pub struct BurnVentura {}

// TODO
#[derive(Accounts)]
pub struct FetchIgnisMarketData {
    // pub market_oracle: Account<'info>,
}

// TODO
#[derive(Accounts)]
pub struct FetchVenturaMarketData {}
