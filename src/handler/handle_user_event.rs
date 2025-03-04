use anyhow::Result;
use hyperliquid_rust_sdk::TradeInfo;

pub async fn handle_user_event(trade_infos: Vec<TradeInfo>) -> Result<()> {
    for (index, trade) in trade_infos.iter().enumerate() {
        let is_possible_spot = match trade.dir.as_str() {
            "Buy" | "Sell" => true,
            "Close Long" | "Close Short" => false,
            "Open Long" | "Open Short" => false,
            _ => {
                println!("  未知交易类型: {}", trade.dir);
                false
            }
        };
        if is_possible_spot {
            println!("==========================");
            println!("聪明钱现货交易: {} {}", trade.dir, trade.coin);
            println!("Price: {}", trade.px);
            println!("amount: {}", trade.sz);
            println!("==========================");
        }
    }
    Ok(())
}
