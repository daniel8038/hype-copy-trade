use anyhow::Result;
use hyperliquid_rust_sdk::{Trade, TradeInfo};

pub async fn handle_trades_event(trade_infos: Vec<Trade>) -> Result<()> {
    // for (index, trade) in trade_infos.iter().enumerate() {
    //     println!("=========================");
    //     println!("  币种: {}", trade.coin);
    //     println!("  方向: {}", trade.side);
    //     println!("  价格: {}", trade.px);
    //     println!("  数量: {}", trade.sz);
    //     println!("  时间: {}", trade.time);
    //     println!("  哈希: {}", trade.hash);
    // }
    Ok(())
}
