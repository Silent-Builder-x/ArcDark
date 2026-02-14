use arcis::*;

#[encrypted]
mod dark_pool {
    use arcis::*;

    pub struct OrderData {
        pub price: u64,
        pub volume: u64,
        pub side: u64,        // 0 = Buy, 1 = Sell
        pub min_exec_qty: u64 // Minimum execution quantity (protects privacy, prevents small-scale probing)
    }

    pub struct TradeResult {
        pub is_executed: u64, // 1 = Success, 0 = Fail
        pub exec_price: u64,  // Execution price
        pub exec_volume: u64, // Execution volume
    }

    /// Core matching logic for the dark pool
    /// Logic: Verify direction, price crossing, minimum quantity constraints, and calculate the midpoint price
    #[instruction]
    pub fn execute_dark_match(
        maker_ctxt: Enc<Shared, OrderData>,
        taker_ctxt: Enc<Shared, OrderData>
    ) -> Enc<Shared, TradeResult> {
        let maker = maker_ctxt.to_arcis();
        let taker = taker_ctxt.to_arcis();

        // 1. Verify direction: must be one buy and one sell (0+1=1)
        // If it is 0+0 or 1+1, it is an invalid match
        let valid_sides = (maker.side + taker.side) == 1;

        // 2. Identify buyer and seller
        // Use Mux (Multiplexer) for dynamic allocation, no need to expose who is the Maker
        let is_maker_buy = maker.side == 0;
        
        let buy_price = if is_maker_buy { maker.price } else { taker.price };
        let sell_price = if is_maker_buy { taker.price } else { maker.price };
        
        // 3. Price crossing check: buy price must be >= sell price
        let price_match = buy_price >= sell_price;

        // 4. Calculate the maximum executable volume (take the smaller value of the two)
        let exec_vol = if maker.volume < taker.volume { maker.volume } else { taker.volume };

        // 5. Minimum execution quantity check (AON - All Or None variant)
        // Only execute if the trade volume meets both parties' "minimum threshold" to prevent privacy leaks
        let min_fill_satisfied = (exec_vol >= maker.min_exec_qty) && (exec_vol >= taker.min_exec_qty);

        // 6. Final determination
        let can_trade = valid_sides && price_match && min_fill_satisfied;

        // 7. Calculate the fair execution price (midpoint price)
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

        // Return the encrypted result to the Maker (order placer)
        maker_ctxt.owner.from_arcis(result)
    }
}