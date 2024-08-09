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
        initial_ignis_supply: u64,
        initial_ventura_supply: u64,
    ) -> Result<()> {
        // Initialise ignis mint
        token::initialize_mint(
            ctx.accounts.initialise_mint_ctx(Coin::Ignis),
            6,                                              // ignis decimal precision
            ctx.accounts.signing_pda.to_account_info().key, // set the mint_authority to the signing_pda
            Some(ctx.accounts.signing_pda.to_account_info().key),
        )?;

        // Initialize ventura mint
        token::initialize_mint(
            ctx.accounts.initialise_mint_ctx(Coin::Ventura),
            6,                                              // ventura decimal precision
            ctx.accounts.signing_pda.to_account_info().key, // set the mint_authority to the signing_pda
            Some(ctx.accounts.signing_pda.to_account_info().key),
        )?;

        // Initialize ignis reserve
        token::initialize_account(ctx.accounts.initialise_reserve_ctx(Coin::Ignis))?;

        // Initialize ventura reserve
        token::initialize_account(ctx.accounts.initialise_reserve_ctx(Coin::Ventura))?;

        // Mint ignis to the reserve
        crate::mint_to(
            &ctx.accounts.token_program,
            &ctx.accounts.ignis_mint,
            &ctx.accounts.ignis_reserve,
            &ctx.accounts.signing_pda,
            ctx.bumps.signing_pda,
            initial_ignis_supply,
        )?;

        // Mint ventura to the reserve
        crate::mint_to(
            &ctx.accounts.token_program,
            &ctx.accounts.ventura_mint,
            &ctx.accounts.ventura_reserve,
            &ctx.accounts.signing_pda,
            ctx.bumps.signing_pda,
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
        ignis_stablecoin.reserve_wallet = ctx.accounts.reserve_wallet.key();

        // Initialize ventura coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.reserve_amount = initial_ventura_supply;
        ventura_coin.circulating_supply = 0;
        ventura_coin.mint = ctx.accounts.ventura_mint.to_account_info().key();
        ventura_coin.ventura_reserve = ctx.accounts.ventura_reserve.to_account_info().key();
        ventura_coin.reserve_wallet = ctx.accounts.reserve_wallet.key();

        Ok(())
    }

    pub fn redeem_ignis(ctx: Context<Redeem>, amount: u64) -> Result<()> {
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
        token::burn(ctx.accounts.burn_user_coin_ctx(Coin::Ignis), amount)?;

        // Mint ventura to the user's account
        token::mint_to(
            ctx.accounts
                .mint_to_user_ctx(Coin::Ventura)
                .with_signer(&[&[&[ctx.bumps.signing_pda][..]][..]]),
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

    pub fn redeem_ventura(ctx: Context<Redeem>, amount: u64) -> Result<()> {
        // TODO: Ensure that ventura is listed on the market and fetch the latest market data
        // TODO: Calculate the equivalent ignis_amount using the market data
        let ignis_amount = 2; // TODO: replace this placeholder with code to compute this value

        // Burn ventura from the user's account
        token::burn(ctx.accounts.burn_user_coin_ctx(Coin::Ventura), amount)?;

        // Mint ignis to the user's account
        token::mint_to(
            ctx.accounts
                .mint_to_user_ctx(Coin::Ignis)
                .with_signer(&[&[&[ctx.bumps.signing_pda][..]][..]]),
            ignis_amount,
        )?;

        // Update ignis stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.circulating_supply += ignis_amount;

        // Update ventura coin properties
        let ventura_coin = &mut ctx.accounts.ventura_coin;
        ventura_coin.circulating_supply -= amount;
        Ok(())
    }

    pub fn mint_ignis_to(ctx: Context<MintIgnisTo>, amount: u64) -> Result<()> {
        // Mint ignis to the reserve
        crate::mint_to(
            &ctx.accounts.token_program,
            &ctx.accounts.ignis_mint,
            &ctx.accounts.to,
            &ctx.accounts.signing_pda,
            ctx.bumps.signing_pda,
            amount,
        )?;

        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        let to = &ctx.accounts.to;

        // Update stablecoin properties
        if to.key() == ignis_stablecoin.ignis_reserve {
            ignis_stablecoin.reserve_amount += amount;
        } else {
            ignis_stablecoin.circulating_supply += amount;
        }

        Ok(())
    }

    pub fn mint_ventura_to(ctx: Context<MintVenturaTo>, amount: u64) -> Result<()> {
        // Mint ventura to the target account
        crate::mint_to(
            &ctx.accounts.token_program,
            &ctx.accounts.ventura_mint,
            &ctx.accounts.to,
            &ctx.accounts.signing_pda,
            ctx.bumps.signing_pda,
            amount,
        )?;

        let ventura_coin = &mut ctx.accounts.ventura_coin;
        let to = &ctx.accounts.to;

        // Update coin properties
        if to.key() == ventura_coin.ventura_reserve {
            ventura_coin.reserve_amount += amount;
        } else {
            ventura_coin.circulating_supply += amount;
        }

        Ok(())
    }

    pub fn burn_reserve_ignis(ctx: Context<BurnReserveIgnis>, amount: u64) -> Result<()> {
        // Burn ignis from the reserve
        crate::burn_from_reserve(
            &ctx.accounts.token_program,
            &ctx.accounts.ignis_mint,
            &ctx.accounts.ignis_reserve,
            &ctx.accounts.signing_pda,
            ctx.bumps.signing_pda,
            amount,
        )?;

        // Update stablecoin properties
        let ignis_stablecoin = &mut ctx.accounts.ignis_stablecoin;
        ignis_stablecoin.reserve_amount -= amount;
        Ok(())
    }

    pub fn burn_reserve_ventura(ctx: Context<BurnReserveVentura>, amount: u64) -> Result<()> {
        // Burn ventura from the reserve
        crate::burn_from_reserve(
            &ctx.accounts.token_program,
            &ctx.accounts.ventura_mint,
            &ctx.accounts.ventura_reserve,
            &ctx.accounts.signing_pda,
            ctx.bumps.signing_pda,
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
        coin: Coin,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeMint<'info>> {
        let token_program = self.token_program.to_account_info();
        let mint: &Account<'info, Mint>;

        match coin {
            Coin::Ignis => mint = &self.ignis_mint,
            Coin::Ventura => mint = &self.ventura_mint,
        }

        let cpi_accounts = InitializeMint {
            mint: mint.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(token_program, cpi_accounts)
    }

    pub fn initialise_reserve_ctx(
        &self,
        coin: Coin,
    ) -> CpiContext<'_, '_, '_, 'info, InitializeAccount<'info>> {
        let token_program = self.token_program.to_account_info();
        let mint: &Account<'info, Mint>;
        let reserve: &Account<'info, TokenAccount>;

        match coin {
            Coin::Ignis => {
                mint = &self.ignis_mint;
                reserve = &self.ignis_reserve;
            }
            Coin::Ventura => {
                mint = &self.ventura_mint;
                reserve = &self.ventura_reserve;
            }
        }

        let cpi_accounts = InitializeAccount {
            account: reserve.to_account_info(),
            mint: mint.to_account_info(),
            authority: self.signing_pda.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(token_program, cpi_accounts)
    }
}

impl<'info> Redeem<'info> {
    pub fn burn_user_coin_ctx(&self, coin: Coin) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let token_program = self.token_program.to_account_info();
        let mint: &Account<'info, Mint>;
        let user_token_account: &Account<'info, TokenAccount>;

        match coin {
            Coin::Ignis => {
                mint = &self.ignis_mint;
                user_token_account = &self.user_ignis_account;
            }
            Coin::Ventura => {
                mint = &self.ventura_mint;
                user_token_account = &self.user_ventura_account;
            }
        }

        let cpi_accounts = Burn {
            mint: mint.to_account_info(),
            from: user_token_account.to_account_info(),
            authority: self.user.to_account_info(),
        };

        CpiContext::new(token_program, cpi_accounts)
    }

    pub fn mint_to_user_ctx(&self, coin: Coin) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let token_program = self.token_program.to_account_info();
        let mint: &Account<'info, Mint>;
        let user_token_account: &Account<'info, TokenAccount>;

        match coin {
            Coin::Ignis => {
                mint = &self.ignis_mint;
                user_token_account = &self.user_ignis_account;
            }
            Coin::Ventura => {
                mint = &self.ventura_mint;
                user_token_account = &self.user_ventura_account;
            }
        }

        let cpi_accounts = MintTo {
            authority: self.signing_pda.to_account_info(),
            mint: mint.to_account_info(),
            to: user_token_account.to_account_info(),
        };

        CpiContext::new(token_program, cpi_accounts)
    }
}

pub fn mint_to<'info>(
    token_program: &Program<'info, Token>,
    token_mint: &Account<'info, Mint>,
    to: &Account<'info, TokenAccount>,
    signing_pda: &UncheckedAccount<'info>,
    signing_pda_bump: u8,
    amount: u64,
) -> Result<()> {
    let token_program = token_program.to_account_info();
    let cpi_accounts = MintTo {
        authority: signing_pda.to_account_info(),
        mint: token_mint.to_account_info(),
        to: to.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_program, cpi_accounts);

    token::mint_to(
        cpi_ctx.with_signer(&[&[&[signing_pda_bump][..]][..]]),
        amount,
    )
}

pub fn burn_from_reserve<'info>(
    token_program: &Program<'info, Token>,
    token_mint: &Account<'info, Mint>,
    token_reserve: &Account<'info, TokenAccount>,
    signing_pda: &UncheckedAccount<'info>,
    signing_pda_bump: u8,
    amount: u64,
) -> Result<()> {
    let token_program = token_program.to_account_info();
    let cpi_accounts = Burn {
        authority: signing_pda.to_account_info(),
        mint: token_mint.to_account_info(),
        from: token_reserve.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(token_program, cpi_accounts);

    token::burn(
        cpi_ctx.with_signer(&[&[&[signing_pda_bump][..]][..]]),
        amount,
    )
}

#[derive(Accounts)]
pub struct Initialise<'info> {
    // Change this to include space for other fields
    #[account(init, payer = reserve_wallet, space = 8 + 32 + 16 + 8 + 8 + 32 + 32 + 8 + 32, seeds=[b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(init, payer = reserve_wallet, space = 8 + 32 + 16 + 8 + 8 + 32 + 32 + 32, seeds=[b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(init, payer = reserve_wallet, space = Mint::LEN)]
    pub ignis_mint: Account<'info, Mint>,
    #[account(init, payer = reserve_wallet, space = Mint::LEN)]
    pub ventura_mint: Account<'info, Mint>,
    #[account(init, payer = reserve_wallet, space = TokenAccount::LEN)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(init, payer = reserve_wallet, space = TokenAccount::LEN)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    // The address constraint ensures that only the predefined reserve wallet can authorise this instruction
    #[account(mut, address = Pubkey::from_str("52Ygg62kTvXgurKkyezpToHGvmU51CJxLXoEoZ25HnMm").unwrap())]
    pub reserve_wallet: Signer<'info>,
    /// CHECK: PDA is generated to give this program signing authority over the mint and reserve accounts
    #[account(seeds=[], bump)]
    pub signing_pda: UncheckedAccount<'info>,
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
    #[account(seeds=[], bump)]
    pub signing_pda: UncheckedAccount<'info>,
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintIgnisTo<'info> {
    #[account(mut, has_one = reserve_wallet, seeds = [b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, token::mint = ignis_stablecoin.mint)]
    pub to: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    /// CHECK: used as a signing PDA to authorize coin minting
    #[account(seeds=[], bump)]
    pub signing_pda: UncheckedAccount<'info>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_wallet: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnReserveIgnis<'info> {
    #[account(mut, has_one = reserve_wallet, seeds = [b"ignis_stablecoin"], bump)]
    pub ignis_stablecoin: Account<'info, IgnisStablecoin>,
    #[account(mut, address = ignis_stablecoin.ignis_reserve)]
    pub ignis_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ignis_stablecoin.mint)]
    pub ignis_mint: Account<'info, Mint>,
    /// CHECK: used as a signing PDA to authorize coin minting
    #[account(seeds=[], bump)]
    pub signing_pda: UncheckedAccount<'info>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_wallet: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintVenturaTo<'info> {
    #[account(mut, has_one = reserve_wallet, seeds = [b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(mut, token::mint = ventura_coin.mint)]
    pub to: Account<'info, TokenAccount>,
    #[account(mut, address = ventura_coin.mint)]
    pub ventura_mint: Account<'info, Mint>,
    /// CHECK: used as a signing PDA to authorize coin minting
    #[account(seeds=[], bump)]
    pub signing_pda: UncheckedAccount<'info>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_wallet: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnReserveVentura<'info> {
    #[account(mut, has_one = reserve_wallet, seeds = [b"ventura_coin"], bump)]
    pub ventura_coin: Account<'info, VenturaCoin>,
    #[account(mut, address = ventura_coin.ventura_reserve)]
    pub ventura_reserve: Account<'info, TokenAccount>,
    #[account(mut, address = ventura_coin.mint)]
    pub ventura_mint: Account<'info, Mint>,
    /// CHECK: used as a signing PDA to authorize coin minting
    #[account(seeds=[], bump)]
    pub signing_pda: UncheckedAccount<'info>,
    // This must satisfy the has_one constraint on ignis_stablecoin
    pub reserve_wallet: Signer<'info>,
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
    pub reserve_wallet: Pubkey, // signing authority for the reserve
}

#[account]
pub struct VenturaCoin {
    // TODO (low priority): Include name, symbol, image and other metadata
    pub reserve_amount: u64, // measured in microventura (millionths of ventura)
    pub circulating_supply: u64, // excludes reserve_amount, measured in microventura
    pub mint: Pubkey,        // mint account address
    pub ventura_reserve: Pubkey, // address of the ventura token account that belongs to the reserve
    pub reserve_wallet: Pubkey, // signing authority for the reserve
}

pub enum Coin {
    Ignis,
    Ventura,
}
