use std::{sync::Arc, time::Duration};

use anyhow::Result;
use hyperliquid_rust_sdk::{
    ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus, TradeInfo,
};
use tokio::time::sleep;

use crate::constants::TRADE_AMOUNT_USDT;

pub async fn handle_user_event(
    trade_infos: Vec<TradeInfo>,
    exchange_client: Arc<ExchangeClient>,
) -> Result<()> {
    for (_index, trade) in trade_infos.iter().enumerate() {
        let trade_type = trade.dir.as_str();
        match trade_type {
            "Buy" => {
                println!("===============聪明现货买入信息==================");
                println!(
                    "聪明钱进行现货买入订单: 代币：{} 价格：{} 数量: {}",
                    trade.coin, trade.px, trade.sz
                );
                execute_spot_buy_order(&trade, exchange_client.clone()).await?;
            }
            "Sell" => {
                println!("===============聪明现货卖出信息==================");
                println!(
                    "聪明钱进行现货卖出订单: 代币：{} 价格：{} 数量: {}",
                    trade.coin, trade.px, trade.sz
                );
                execute_spot_sell_order(&trade, exchange_client.clone()).await?;
            }
            // ......
            "Close Long" => {
                // println!("聪明钱 Close Long: {} {}", trade.dir, trade.coin);
            }
            "Close Short" => {
                // println!("聪明钱 Close Short: {} {}", trade.dir, trade.coin);
            }
            "Open Long" => {
                // println!("聪明钱 Open Long: {} {}", trade.dir, trade.coin);
            }
            "Open Short" => {
                // println!("聪明钱 Open Short: {} {}", trade.dir, trade.coin);
            }
            _ => {
                println!("未知类型");
            }
        }
    }
    Ok(())
}
async fn execute_spot_buy_order(
    trade: &TradeInfo,
    exchange_client: Arc<ExchangeClient>,
) -> Result<()> {
    println!("执行现货买入 {} 跟单", trade.coin);
    let order = ClientOrderRequest {
        asset: trade.coin.to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: trade.px.parse::<f64>().unwrap() * 1.1,
        // 100 U
        sz: TRADE_AMOUNT_USDT / trade.px.parse::<f64>().unwrap(),
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("Error: {status:?}"),
    };
    println!("跟单购买成功： 订单id {}", oid);
    Ok(())
}

async fn execute_spot_sell_order(
    trade: &TradeInfo,
    exchange_client: Arc<ExchangeClient>,
) -> Result<()> {
    println!("执行现货卖出 {} 跟单", trade.coin);
    todo!();
    Ok(())
}
