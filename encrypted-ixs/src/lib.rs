use arcis::*;

#[encrypted]
pub mod arcis_circuits {
    use arcis::*;

    pub struct PoolState {
        pub reserve_a: u64,
        pub reserve_b: u64,
        pub fee_numerator: u64,
        pub fee_denominator: u64,
        pub is_initialized: bool,
    }

    pub struct SwapRequest {
        pub user: SerializedSolanaPublicKey,
        pub is_a_to_b: bool,
        pub amount_in: u64,
        pub min_amount_out: u64,
    }

    pub struct SwapResult {
        pub user: SerializedSolanaPublicKey,
        pub success: bool,
        pub amount_out: u64,
        pub new_pool_state: PoolState,
    }

    #[instruction]
    pub fn init_pool(
        initial_reserve_a_ctxt: Enc<Shared, u64>,
        initial_reserve_b_ctxt: Enc<Shared, u64>,
    ) -> Enc<Mxe, PoolState> {
        let initial_a = initial_reserve_a_ctxt.to_arcis();
        let initial_b = initial_reserve_b_ctxt.to_arcis();

        let initial_state = PoolState {
            reserve_a: initial_a,
            reserve_b: initial_b,
            fee_numerator: 0,      // 暗池承兑免手续费
            fee_denominator: 10000,
            is_initialized: true,
        };
        Mxe::get().from_arcis(initial_state)
    }

    // 🚀 降维打击版：极速暗池大宗承兑 (无除法，极低 CU 消耗)
    #[instruction]
    pub fn execute_swap(
        request_ctxt: Enc<Shared, SwapRequest>,
        pool_state_ctxt: Enc<Mxe, PoolState>,
    ) -> Enc<Mxe, SwapResult> {
        let req = request_ctxt.to_arcis();
        let mut state = pool_state_ctxt.to_arcis();

        let mut result = SwapResult {
            user: req.user,
            success: false,
            amount_out: 0,
            new_pool_state: PoolState {
                reserve_a: state.reserve_a,
                reserve_b: state.reserve_b,
                fee_numerator: state.fee_numerator,
                fee_denominator: state.fee_denominator,
                is_initialized: state.is_initialized,
            },
        };

        if state.is_initialized && req.amount_in > 0 {
            let amount_out = req.amount_in; // 1:1 暗箱成交

            if req.is_a_to_b {
                if amount_out >= req.min_amount_out && amount_out <= state.reserve_b {
                    result.success = true;
                    result.amount_out = amount_out;
                    state.reserve_a += req.amount_in;
                    state.reserve_b -= amount_out;
                    result.new_pool_state.reserve_a = state.reserve_a;
                    result.new_pool_state.reserve_b = state.reserve_b;
                }
            } else {
                if amount_out >= req.min_amount_out && amount_out <= state.reserve_a {
                    result.success = true;
                    result.amount_out = amount_out;
                    state.reserve_b += req.amount_in;
                    state.reserve_a -= amount_out;
                    result.new_pool_state.reserve_a = state.reserve_a;
                    result.new_pool_state.reserve_b = state.reserve_b;
                }
            }
        }
        Mxe::get().from_arcis(result)
    }
}