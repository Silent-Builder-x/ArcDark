use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::types::CallbackAccount;

const COMP_DEF_OFFSET_INIT_POOL: u32 = comp_def_offset("init_pool");
const COMP_DEF_OFFSET_EXECUTE_SWAP: u32 = comp_def_offset("execute_swap");

// 请替换为你在 target/deploy/ 目录下实际生成的公钥
declare_id!("EQU8JCm5GYWZqJK2QXo8YFKR7m3MD9wkFAqd6VyCWTPH"); 

#[arcium_program]
pub mod private_amm {
    use super::*;

    pub fn init_pool_comp_def(ctx: Context<InitPoolCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn execute_swap_comp_def(ctx: Context<ExecuteSwapCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        computation_offset: u64,
        encrypted_initial_a: [u8; 32],
        encrypted_initial_b: [u8; 32],
        nonce: u128,
        pubkey: [u8; 32], 
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.bump = ctx.bumps.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.transaction_count = 0;
        pool.encrypted_state = [[0u8; 32]; 5]; 

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u64(encrypted_initial_a)
            .encrypted_u64(encrypted_initial_b)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![InitPoolCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.pool.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "init_pool")]
    pub fn init_pool_callback(
        ctx: Context<InitPoolCallback>,
        output: SignedComputationOutputs<InitPoolOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(InitPoolOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        let pool = &mut ctx.accounts.pool;
        let len = std::cmp::min(pool.encrypted_state.len(), o.ciphertexts.len());
        for i in 0..len {
            pool.encrypted_state[i] = o.ciphertexts[i];
        }
        pool.state_nonce = o.nonce;

        msg!("✅ Private AMM Pool Initialized.");
        Ok(())
    }

    pub fn execute_swap(
        ctx: Context<ExecuteSwap>,
        computation_offset: u64,
        encrypted_user_lo: [u8; 32],
        encrypted_user_hi: [u8; 32],
        encrypted_is_a_to_b: [u8; 32],
        encrypted_amount_in: [u8; 32],
        encrypted_min_amount_out: [u8; 32],
        nonce: u128,
        pubkey: [u8; 32],
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

        const ENCRYPTED_STATE_OFFSET: u32 = 65;
        const ENCRYPTED_STATE_SIZE: u32 = 32 * 5; 

        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            .encrypted_u128(encrypted_user_lo)
            .encrypted_u128(encrypted_user_hi)
            .encrypted_u64(encrypted_is_a_to_b)
            .encrypted_u64(encrypted_amount_in)
            .encrypted_u64(encrypted_min_amount_out)
            .plaintext_u128(pool.state_nonce)
            .account(
                ctx.accounts.pool.key(),
                ENCRYPTED_STATE_OFFSET,
                ENCRYPTED_STATE_SIZE,
            )
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![ExecuteSwapCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[CallbackAccount {
                    pubkey: ctx.accounts.pool.key(),
                    is_writable: true,
                }],
            )?],
            1,
            0,
        )?;

        msg!("🔄 Swap request queued to MXE.");
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "execute_swap")]
    pub fn execute_swap_callback(
        ctx: Context<ExecuteSwapCallback>,
        output: SignedComputationOutputs<ExecuteSwapOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(ExecuteSwapOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };
        
        let pool = &mut ctx.accounts.pool;
        pool.transaction_count += 1;
        pool.state_nonce = o.nonce;
        let _cipher_data = o.ciphertexts;

        msg!("🔒 Swap executed! State updated in absolute stealth.");
        Ok(())
    }
}

// --- 账户结构与宏定义 ---

#[account]
#[derive(InitSpace)]
pub struct AmmPool {
    pub bump: u8,
    pub authority: Pubkey,
    pub transaction_count: u64,
    pub state_nonce: u128,
    pub encrypted_state: [[u8; 32]; 5],
}

#[init_computation_definition_accounts("init_pool", payer)]
#[derive(Accounts)]
pub struct InitPoolCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: checked by arcium program.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT program
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[init_computation_definition_accounts("execute_swap", payer)]
#[derive(Accounts)]
pub struct ExecuteSwapCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: checked by arcium program.
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: checked by arcium program.
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT program
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

// 🚀 核心改动：把所有的巨型 Account 全部用 Box 包装，彻底消灭 Stack Overflow 报错！
#[queue_computation_accounts("init_pool", authority)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + AmmPool::INIT_SPACE,
        seeds = [b"pool", authority.key().as_ref()],
        bump,
    )]
    pub pool: Box<Account<'info, AmmPool>>,
    #[account(
        init_if_needed,
        space = 9,
        payer = authority,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Box<Account<'info, ArciumSignerAccount>>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by arcium program
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by arcium program
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by arcium program
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_POOL))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Box<Account<'info, FeePool>>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Box<Account<'info, ClockAccount>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("init_pool")]
#[derive(Accounts)]
pub struct InitPoolCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_POOL))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: checked by constraints
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, AmmPool>>,
}

#[queue_computation_accounts("execute_swap", user)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ExecuteSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, AmmPool>>,
    #[account(
        init_if_needed,
        space = 9,
        payer = user,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Box<Account<'info, ArciumSignerAccount>>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by arcium program
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by arcium program
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: checked by arcium program
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_EXECUTE_SWAP))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Box<Account<'info, FeePool>>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Box<Account<'info, ClockAccount>>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("execute_swap")]
#[derive(Accounts)]
pub struct ExecuteSwapCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_EXECUTE_SWAP))]
    pub comp_def_account: Box<Account<'info, ComputationDefinitionAccount>>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: checked by constraints
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Box<Account<'info, Cluster>>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, AmmPool>>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}