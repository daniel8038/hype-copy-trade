use anyhow::Result;
use ethers::types::H160;
use hyperliquid_rust_sdk::{
    ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, ExchangeDataStatus,
    ExchangeResponseStatus, InfoClient, MarketOrderParams, TradeInfo,
};
use std::{str::FromStr, sync::Arc};

use crate::constants::{MT_ADDRESS, TRADE_AMOUNT_USDT};

pub async fn handle_user_event(
    trade_infos: Vec<TradeInfo>,
    exchange_client: Arc<ExchangeClient>,
    query_client: Arc<InfoClient>,
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
                execute_spot_market_buy_order(&trade, exchange_client.clone()).await?;
                // 限价单 可以挂上止盈止损单
                // execute_spot_limit_sell_order(&trade, exchange_client.clone()).await?;
            }
            "Sell" => {
                println!("===============聪明现货卖出信息==================");
                println!(
                    "聪明钱进行现货卖出订单: 代币：{} 价格：{} 数量: {}",
                    trade.coin, trade.px, trade.sz
                );
                execute_spot_market_sell_order(
                    &trade,
                    exchange_client.clone(),
                    query_client.clone(),
                )
                .await?;
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
async fn execute_spot_market_buy_order(
    trade: &TradeInfo,
    exchange_client: Arc<ExchangeClient>,
) -> Result<()> {
    println!("执行现货买入 {} 跟单", trade.coin);
    let market_open_params = MarketOrderParams {
        asset: &trade.coin,
        is_buy: true,
        sz: TRADE_AMOUNT_USDT / trade.px.parse::<f64>().unwrap(),
        px: None,
        slippage: Some(0.05),
        cloid: None,
        wallet: None,
    };

    let response = exchange_client
        .market_open(market_open_params)
        .await
        .unwrap();

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
    println!("--市价单--跟单购买成功： 订单id {}", oid);
    Ok(())
}
async fn execute_spot_market_sell_order(
    trade: &TradeInfo,
    exchange_client: Arc<ExchangeClient>,
    query_client: Arc<InfoClient>,
) -> Result<()> {
    println!("执行现货买入 {} 跟单", trade.coin);
    let my_all_token_balances = query_client
        .user_token_balances(H160::from_str(MT_ADDRESS).unwrap())
        .await?;
    let current_token_balance = my_all_token_balances
        .balances
        .iter()
        .find(|balance| balance.coin == trade.coin)
        .ok_or_else(|| anyhow::anyhow!("未找到代币 {} 的余额", trade.coin))?;
    // Market open order
    let market_open_params = MarketOrderParams {
        asset: &trade.coin,
        is_buy: false,
        sz: current_token_balance.total.parse::<f64>().unwrap(),
        px: None,
        slippage: Some(0.05), // 1% slippage
        cloid: None,
        wallet: None,
    };

    let response = exchange_client
        .market_open(market_open_params)
        .await
        .unwrap();

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
    println!("--市价单--跟卖出成功： 订单id {}", oid);
    Ok(())
}
#[warn(dead_code)]
async fn execute_spot_limit_buy_order(
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
        // Alo 挂单  Ioc 立即成交  否则失败 Gtc 订单保持有效直到被取消或完全成交
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Alo".to_string(),
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
    println!("--限价单--跟单购买成功： 订单id {}", oid);
    Ok(())
}

#[warn(dead_code)]
async fn execute_spot_limit_sell_order(
    trade: &TradeInfo,
    exchange_client: Arc<ExchangeClient>,
) -> Result<()> {
    println!("执行现货卖出 {} 跟单", trade.coin);

    todo!();
    Ok(())
}
