use arcis::*;

#[encrypted]
mod dark_pool {
    use arcis::*;

    pub struct OrderData {
        pub price: u64,
        pub volume: u64,
        pub side: u64,        // 0 = Buy (买), 1 = Sell (卖)
        pub min_exec_qty: u64 // 最小执行数量 (保护隐私，防止微量钓鱼)
    }

    pub struct TradeResult {
        pub is_executed: u64, // 1 = Success, 0 = Fail
        pub exec_price: u64,  // 执行价格
        pub exec_volume: u64, // 执行数量
    }

    /// 暗池核心撮合指令
    /// 逻辑：验证方向、价格交叉、最小数量约束，并计算中间价
    #[instruction]
    pub fn execute_dark_match(
        maker_ctxt: Enc<Shared, OrderData>,
        taker_ctxt: Enc<Shared, OrderData>
    ) -> Enc<Shared, TradeResult> {
        let maker = maker_ctxt.to_arcis();
        let taker = taker_ctxt.to_arcis();

        // 1. 验证方向：必须是一买一卖 (0+1=1)
        // 如果是 0+0 或 1+1，则为无效匹配
        let valid_sides = (maker.side + taker.side) == 1;

        // 2. 识别买方和卖方
        // 使用 Mux (Multiplexer) 动态分配，无需暴露谁是 Maker
        let is_maker_buy = maker.side == 0;
        
        let buy_price = if is_maker_buy { maker.price } else { taker.price };
        let sell_price = if is_maker_buy { taker.price } else { maker.price };
        
        // 3. 价格交叉检查：买价必须 >= 卖价
        let price_match = buy_price >= sell_price;

        // 4. 计算最大可成交量 (取两者较小值)
        let exec_vol = if maker.volume < taker.volume { maker.volume } else { taker.volume };

        // 5. 最小执行数量检查 (AON - All Or None 变体)
        // 只有当成交量满足双方的“最小门槛”时才成交，防止隐私泄露
        let min_fill_satisfied = (exec_vol >= maker.min_exec_qty) && (exec_vol >= taker.min_exec_qty);

        // 6. 最终判定
        let can_trade = valid_sides && price_match && min_fill_satisfied;

        // 7. 计算公平执行价 (中间价)
        let mid_price = (buy_price + sell_price) / 2;

        let result = if can_trade {
            TradeResult {
                is_executed: 1,
                exec_price: mid_price,
                exec_volume: exec_vol,
            }
        } else {
            TradeResult {
                is_executed: 0,
                exec_price: 0,
                exec_volume: 0,
            }
        };

        // 结果加密返回给 Maker (挂单方)
        maker_ctxt.owner.from_arcis(result)
    }
}