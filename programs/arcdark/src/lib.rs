use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_MATCH: u32 = comp_def_offset("execute_dark_match");

declare_id!("BrwrywS88APv6LmBkniYNgcFMq9fYKXnqD2ZUW9HcPUf");

#[arcium_program]
pub mod arcdark {
    use super::*;

    pub fn init_pool_config(ctx: Context<InitPoolCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// [新增] 挂单 (Place Order)
    /// 用户创建一个持久化的链上订单账户，存储加密后的意图
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        encrypted_data: [[u8; 32]; 4], // [Price, Volume, Side, MinFill]
    ) -> Result<()> {
        let order = &mut ctx.accounts.order;
        order.owner = ctx.accounts.owner.key();
        order.encrypted_data = encrypted_data;
        order.is_active = true;
        order.bump = ctx.bumps.order;
        
        msg!("Dark Order Placed. ID: {}", order.key());
        Ok(())
    }

    /// [升级] 执行撮合 (Execute Match)
    /// 传入两个链上订单账户 (Maker, Taker) 进行原子匹配
    pub fn execute_match(
        ctx: Context<ExecuteMatch>,
        computation_offset: u64,
        pubkey: [u8; 32], // 结果重加密公钥
        nonce: u128,
    ) -> Result<()> {
        let accounts = &mut ctx.accounts.computation;
        accounts.sign_pda_account.bump = ctx.bumps.computation.sign_pda_account;
        
        // 构建 MPC 参数
        // 严格对应电路输入: fn execute_dark_match(maker, taker)
        let mut builder = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce);

        // 1. 读取 Maker 订单加密数据
        for shard in &ctx.accounts.maker_order.encrypted_data {
            builder = builder.encrypted_u64(*shard);
        }

        // 2. 读取 Taker 订单加密数据
        for shard in &ctx.accounts.taker_order.encrypted_data {
            builder = builder.encrypted_u64(*shard);
        }

        queue_computation(
            accounts,
            computation_offset,
            builder.build(),
            vec![ExecuteDarkMatchCallback::callback_ix(
                computation_offset,
                &accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        
        msg!("Match Request Queued. Maker: {}, Taker: {}", 
             ctx.accounts.maker_order.key(), 
             ctx.accounts.taker_order.key());
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "execute_dark_match")]
    pub fn execute_dark_match_callback(
        ctx: Context<ExecuteDarkMatchCallback>,
        output: SignedComputationOutputs<ExecuteDarkMatchOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(ExecuteDarkMatchOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        // 解析结果: { is_executed, exec_price, exec_volume }
        let status_bytes: [u8; 8] = o.ciphertexts[0][0..8].try_into().unwrap();
        let is_executed = u64::from_le_bytes(status_bytes) == 1;

        if is_executed {
            msg!("✅ TRADE EXECUTED via Dark Pool!");
            // 在真实场景中，这里会标记订单为 Closed 或更新余额
            // 由于 MPC 输出是加密的，这里仅通过 Event 通知链下 Relayer 进行结算
        } else {
            msg!("⚠️ Match Failed: Conditions not met.");
        }

        emit!(TradeEvent {
            maker_order: ctx.accounts.computation_account.key(), // 简化，实际需传参
            success: is_executed,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// --- Accounts ---

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        // Space: Disc(8) + Owner(32) + Data(4*32=128) + Bool(1) + Bump(1)
        space = 8 + 32 + 128 + 1 + 1,
        seeds = [b"order", owner.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()[0..4]], // 简单随机种子
        bump
    )]
    pub order: Account<'info, DarkOrder>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct DarkOrder {
    pub owner: Pubkey,
    pub encrypted_data: [[u8; 32]; 4], // Price, Vol, Side, MinFill
    pub is_active: bool,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ExecuteMatch<'info> {
    pub computation: ExecuteMatchBase<'info>,
    
    // 传入两个订单账户
    #[account(constraint = maker_order.is_active)]
    pub maker_order: Account<'info, DarkOrder>,
    #[account(constraint = taker_order.is_active)]
    pub taker_order: Account<'info, DarkOrder>,
}

#[queue_computation_accounts("execute_dark_match", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct ExecuteMatchBase<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>, // Box for stack safety
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Mempool
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Execpool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Comp Acct
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("execute_dark_match")]
#[derive(Accounts)]
pub struct ExecuteDarkMatchCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: Comp Acct
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("execute_dark_match", payer)]
#[derive(Accounts)]
pub struct InitPoolCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: Comp Def
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: LUT
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT Prog
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct TradeEvent {
    pub maker_order: Pubkey,
    pub success: bool,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Aborted")] AbortedComputation,
    #[msg("No Cluster")] ClusterNotSet,
}