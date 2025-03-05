use anyhow::Result;
use hyperliquid_rust_sdk::TradeInfo;

pub async fn handle_user_event(trade_infos: Vec<TradeInfo>) -> Result<()> {
    for (index, trade) in trade_infos.iter().enumerate() {
        let trade_type = trade.dir.as_str();
        match trade_type {
            "Buy" => {
                println!("聪明钱现货 Buy: {} {}", trade.dir, trade.coin);
            }
            "Sell" => {
                println!("聪明钱现货 Sell: {} {}", trade.dir, trade.coin);
            }
            "Close Long" => {
                println!("聪明钱 Close Long: {} {}", trade.dir, trade.coin);
            }
            "Close Short" => {
                println!("聪明钱 Close Short: {} {}", trade.dir, trade.coin);
            }
            "Open Long" => {
                println!("聪明钱 Open Long: {} {}", trade.dir, trade.coin);
            }
            "Open Short" => {
                println!("聪明钱 Open Short: {} {}", trade.dir, trade.coin);
            }
            _ => {
                println!("未知类型");
            }
        }
    }
    Ok(())
}
