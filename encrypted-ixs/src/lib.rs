use arcis::*;

#[encrypted]
mod dark_pool {
    use arcis::*;

    pub struct Order {
        pub price: u64,
        pub volume: u64,
        pub trader_id: u64,
    }

    pub struct MatchBatch {
        pub buy_order: Order,
        pub sell_order: Order,
    }

    pub struct TradeResult {
        pub is_matched: u64,   // 1 为撮合成功, 0 为失败
        pub exec_price: u64,
        pub exec_volume: u64,
        pub winner_id: u64,    // 仅用于演示：撮合后的受益方
    }

    #[instruction]
    pub fn match_orders(
        input_ctxt: Enc<Shared, MatchBatch>
    ) -> Enc<Shared, TradeResult> {
        let input = input_ctxt.to_arcis();
        let buy = input.buy_order;
        let sell = input.sell_order;

        // 核心撮合逻辑：买价 >= 卖价 且 都有量
        let can_match = (buy.price >= sell.price) && (buy.volume > 0) && (sell.volume > 0);

        // 撮合成功时的参数计算
        let mid_price = (buy.price + sell.price) / 2;
        let matched_vol = if buy.volume >= sell.volume { sell.volume } else { buy.volume };

        let result = if can_match {
            TradeResult {
                is_matched: 1u64,
                exec_price: mid_price,
                exec_volume: matched_vol,
                winner_id: buy.trader_id,
            }
        } else {
            TradeResult {
                is_matched: 0u64,
                exec_price: 0u64,
                exec_volume: 0u64,
                winner_id: 0u64,
            }
        };

        input_ctxt.owner.from_arcis(result)
    }
}