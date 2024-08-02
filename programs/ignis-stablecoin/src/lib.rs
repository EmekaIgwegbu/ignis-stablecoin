use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, Burn, InitializeAccount, InitializeMint, Mint, MintTo, Token, TokenAccount,
};
// use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use std::str::FromStr;

declare_id!("Bou8TKf8G9iJQoZMptYtFHrpgvEjG4DTTXo8sxShLsht");

#[program]
pub mod ignis_stablecoin {
    use super::*;

    pub fn initialise(
        ctx: Context<Initialise>,
        bump: u8,
        initial_ignis_supply: u64,
        initial_ventura_supply: u64,
    ) -> Result<()> {
        // Initialise ignis mint
        token::initialize_mint(
            ctx.accounts.initialise_mint_ctx(&ctx.accounts.ignis_mint),
            6,                                                // ignis decimal precision
            ctx.accounts.pda_authority.to_account_info().key, // set the mint_authority to the pda_authority
            Some(ctx.accounts.pda_authority.to_account_info().key),
        )?;

        // Initialize ventura mint
        token::initialize_mint(
            ctx.accounts.initialise_mint_ctx(&ctx.accounts.ventura_mint),
            6,                                                // ventura decimal precision
            ctx.accounts.pda_authority.to_account_info().key, // set the mint_authority to the pda_authority
            Some(ctx.accounts.pda_authority.to_account_info().key),
        )?;

        // Initialize ignis reserve
        token::initialize_account(
            ctx.accounts
                .initialise_reserve_ctx(&ctx.accounts.ignis_mint, &ctx.accounts.ignis_reserve)
                .with_signer(&[&[&[bump][..]][..]]),
        )?;

        // Initialize ventura reserve
        token::initialize_account(
            ctx.accounts
                .initialise_reserve_ctx(&ctx.accounts.ventura_mint, &ctx.accounts.ventura_reserve)
                .with_signer(&[&[&[bump][..]][..]]),
        )?;

        // Mint ignis to the reserve
        token::mint_to(
            ctx.accounts
                .mint_to_reserve_ctx(&ctx.accounts.ignis_mint, &ctx.accounts.ignis_reserve)
                .with_signer(&[&[&[bump][..]][..]]),
            initial_ignis_supply,
        )?;

        // Mint ventura to the reserve
        token::mint_to(
            ctx.accounts
                .mint_to_reserve_ctx(&ctx.accounts.ventura_mint, &ctx.accounts.ventura_reserve)
                .with_signer(&[&[&[bump][..]][..]]),
            initial_ventura_supply,
        )?;

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

    pub fn redeem_ignis(ctx: Context<Redeem>, bump: u8, amount: u64) -> Result<()> {
        // TODO: Ensure that ventura is listed on the market and fetch the latest market data
        // TODO: Calculate the equivalent ventura_amount using the market data

        // let ventura_price_update = &mut ctx.accounts.ventura_price_update;
        // let maximum_age: u64 = 30;
        // let feed_id: [u8; 32] = get_feed_id_from_hex(
        //     "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
        // )?;
        // let ventura_price =
        //     ventura_price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;

        // let ventura_amount = ctx.accounts.ignis_stablecoin.peg
        //     / (ventura_price.price * 10i32.pow(ventura_price.exponent));

        let ventura_amount = 2; // TODO: replace this placeholder with code to compute this value

        // Burn ignis from the user's account
        token::burn(
            burn_ctx(
                &ctx.accounts.token_program,
                &ctx.accounts.user,
                &ctx.accounts.ignis_mint,
                &ctx.accounts.user_ignis_account,
            ),
            amount,
        )?;

        // Mint ventura to the user's account
        token::mint_to(
            mint_to_ctx(
                &ctx.accounts.token_program,
                &ctx.accounts.pda_authority,
                &ctx.accounts.ventura_mint,
                &ctx.accounts.user_ventura_account,
            )
            .with_signer(&[&[&[bump][..]][..]]),
            ventura_amount,
        )?;

        // Update ignis stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.circulating_supply -= amount;

        // Update ventura coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.circulating_supply += ventura_amount;
        Ok(())
    }

    // TODO: High priority
    pub fn redeem_ventura(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    pub fn mint_to_ignis_reserve(
        ctx: Context<MintToIgnisReserve>,
        bump: u8,
        amount: u64,
    ) -> Result<()> {
        // Mint ignis to the reserve
        token::mint_to(
            mint_to_ctx(
                &ctx.accounts.token_program,
                &ctx.accounts.pda_authority,
                &ctx.accounts.ignis_mint,
                &ctx.accounts.ignis_reserve,
            )
            .with_signer(&[&[&[bump][..]][..]]),
            amount,
        )?;

        // Update stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.reserve_amount += amount;
        Ok(())
    }

    pub fn mint_to_ventura_reserve(
        ctx: Context<MintToVenturaReserve>,
        bump: u8,
        amount: u64,
    ) -> Result<()> {
        // Mint ventura to the reserve
        token::mint_to(
            mint_to_ctx(
                &ctx.accounts.token_program,
                &ctx.accounts.pda_authority,
                &ctx.accounts.ventura_mint,
                &ctx.accounts.ventura_reserve,
            )
            .with_signer(&[&[&[bump][..]][..]]),
            amount,
        )?;

        // Update coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.reserve_amount += amount;
        Ok(())
    }

    pub fn burn_reserve_ignis(ctx: Context<BurnReserveIgnis>, amount: u64) -> Result<()> {
        // Burn ignis from the reserve
        token::burn(
            burn_ctx(
                &ctx.accounts.token_program,
                &ctx.accounts.reserve_authority,
                &ctx.accounts.ignis_mint,
                &ctx.accounts.ignis_reserve,
            ),
            amount,
        )?;

        // Update stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.reserve_amount -= amount;
        Ok(())
    }

    pub fn burn_reserve_ventura(ctx: Context<BurnReserveVentura>, amount: u64) -> Result<()> {
        // Burn ventura from the reserve
        token::burn(
            burn_ctx(
                &ctx.accounts.token_program,
                &ctx.accounts.reserve_authority,
                &ctx.accounts.ventura_mint,
                &ctx.accounts.ventura_reserve,
            ),
            amount,
        )?;

        // Update coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.reserve_amount -= amount;
        Ok(())
    }
}

impl<'info> Initialise<'info> {
    pub fn initialise_mint_ctx(
        &self,
        mint: &Account<'info, Mint>,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMint<'info>> {
        let token_program = self.token_program.to_account_info();
        let cpi_accounts = InitializeMint {
            mint: mint.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(token_program, cpi_accounts)
    }

    pub fn initialise_reserve_ctx(
        &self,
        mint: &Account<'info, Mint>,
        reserve: &Account<'info, TokenAccount>,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let token_program = self.token_program.to_account_info();
        let cpi_accounts = InitializeAccount {
            account: reserve.to_account_info(),
            mint: mint.to_account_info(),
            authority: self.pda_authority.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(token_program, cpi_accounts)
    }

    pub fn mint_to_reserve_ctx(
        &self,
        mint: &Account<'info, Mint>,
        reserve: &Account<'info, TokenAccount>,
    ) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        mint_to_ctx(&self.token_program, &self.pda_authority, &mint, &reserve)
    }
}

pub fn burn_ctx<'info>(
    token_program: &Program<'info, Token>,
    authority: &Signer<'info>,
    mint: &Account<'info, Mint>,
    from: &Account<'info, TokenAccount>,
) -> CpiContext<'info, 'info, 'info, 'info, Burn<'info>> {
    let token_program = token_program.to_account_info();
    let cpi_accounts = Burn {
        mint: mint.to_account_info(),
        from: from.to_account_info(),
        authority: authority.to_account_info(),
    };
    CpiContext::new(token_program, cpi_accounts)
}

pub fn mint_to_ctx<'info>(
    token_program: &Program<'info, Token>,
    authority: &UncheckedAccount<'info>,
    mint: &Account<'info, Mint>,
    to: &Account<'info, TokenAccount>,
) -> CpiContext<'info, 'info, 'info, 'info, MintTo<'info>> {
    let token_program = token_program.to_account_info();
    let cpi_accounts = MintTo {
        authority: authority.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
    };
    CpiContext::new(token_program, cpi_accounts)
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    // Change this to include space for other fields
    #[account(init, payer = reserve_authority, space = 8 + 32 + 16 + 8 + 8 + 32 + 32 + 8 + 32, seeds=[b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(init, payer = reserve_authority, space = 8 + 32 + 16 + 8 + 8 + 32 + 32 + 32, seeds=[b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(init, payer = reserve_authority, space = Mint::LEN, seeds=[b"ignis_mint"], bump)]
    pub ignis_mint: Account<'info, Mint>,
    #[account(init, payer = reserve_authority, space = Mint::LEN, seeds=[b"ventura_mint"], bump)]
    pub ventura_mint: Account<'info, Mint>,
    // Create the ignis reserve account with a PDA that will be used to give this program authority over it
    #[account(init, payer = reserve_authority, space = TokenAccount::LEN, seeds=[b"ignis_reserve"], bump)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    // Create the ventura reserve account with a PDA that will be used to give this program authority over it
    #[account(init, payer = reserve_authority, space = TokenAccount::LEN, seeds=[b"ventura_reserve"], bump)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    // The address constraint ensures that only the predefined reserve wallet can authorise this instruction
    #[account(mut, address = Pubkey::from_str("52Ygg62kTvXgurKkyezpToHGvmU51CJxLXoEoZ25HnMm").unwrap())]
    pub reserve_authority: Signer<'info>,
    /// CHECK: PDA is generated to give this program signing authority over the mint and reserve accounts
    pub pda_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
// #[instruction()] // TODO: figure out what this is for. Copied from example in Pyth docs https://docs.pyth.network/price-feeds/use-real-time-data/solana#write-contract-code
pub struct Redeem<'info> {
    #[account(mut, seeds = [b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, seeds = [b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(mut, token::authority = user, token::mint = ignis_stablecoin.mint)]
    pub user_ignis_account: Account<'info, TokenAccount>,
    #[account(mut, token::authority = user, token::mint = ventura_coin.mint)]
    pub user_ventura_account: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    #[account(mut, address = ventura_coin.mint)]
    pub ventura_mint: Account<'info, Mint>,
    // Used to fetch the latest ventura price data
    // pub ventura_price_update: Account<'info, PriceUpdateV2>,
    /// CHECK: used as a signing PDA to authorize coin minting
    pub pda_authority: UncheckedAccount<'info>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintToIgnisReserve<'info> {
    #[account(mut, has_one = reserve_authority, seeds = [b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, address = ignis_stablecoin.ignis_reserve)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    /// CHECK: used as a signing PDA to authorize coin minting
    pub pda_authority: UncheckedAccount<'info>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnReserveIgnis<'info> {
    #[account(mut, has_one = reserve_authority, seeds = [b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, address = ignis_stablecoin.ignis_reserve)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// TODO: low priority
#[derive(Accounts)]
pub struct MintToVenturaReserve<'info> {
    #[account(mut, has_one = reserve_authority, seeds = [b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(mut, address = ventura_coin.ventura_reserve)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ventura_coin.mint)]
    pub ventura_mint: Account<'info, Mint>,
    /// CHECK: used as a signing PDA to authorize coin minting
    pub pda_authority: UncheckedAccount<'info>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

// TODO: low priority
#[derive(Accounts)]
pub struct BurnReserveVentura<'info> {
    #[account(mut, has_one = reserve_authority, seeds = [b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(mut, address = ventura_coin.ventura_reserve)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ventura_coin.mint)]
    pub ventura_mint: Account<'info, Mint>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct IgnisStablecoin {
    // TODO (low priority): Include name, symbol, image and other metadata
    pub reserve_amount: u64, // measured in microignis (millionths of ignis)
    pub circulating_supply: u64, // excludes reserve_amount, measured in microignis
    pub mint: Pubkey,        // mint account address
    pub ignis_reserve: Pubkey, // address of the ignis token account that belongs to the reserve
    pub peg: f64,
    pub reserve_authority: Pubkey, // signing authority representing the reserve
}

#[account]
pub struct VenturaCoin {
    // TODO (low priority): Include name, symbol, image and other metadata
    pub reserve_amount: u64, // measured in microventura (millionths of ventura)
    pub circulating_supply: u64, // excludes reserve_amount, measured in microventura
    pub mint: Pubkey,        // mint account address
    pub ventura_reserve: Pubkey, // address of the ventura token account that belongs to the reserve
    pub reserve_authority: Pubkey, // signing authority representing the reserve
}
