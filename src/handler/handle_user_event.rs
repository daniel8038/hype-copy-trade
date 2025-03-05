use anyhow::{bail, Result};
use ethers::types::H160;
use hyperliquid_rust_sdk::{
    ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, ExchangeDataStatus,
    ExchangeResponseStatus, InfoClient, SpotMeta, TradeInfo,
};
use std::{env, fs, path::Path, str::FromStr, sync::Arc};

use crate::constants::{MY_ADDRESS, TRADE_AMOUNT_USDT};

pub async fn handle_user_event(
    trade_infos: Vec<TradeInfo>,
    exchange_client: Arc<ExchangeClient>,
    query_client: Arc<InfoClient>,
) -> Result<()> {
    let enable_sell = env::var("ENABLE_SELL").unwrap().parse::<bool>().unwrap();
    for (_index, trade) in trade_infos.iter().enumerate() {
        let trade_type = trade.dir.as_str();
        println!("trade_type {}", trade_type);
        match trade_type {
            "Buy" => {
                println!("===============聪明现货买入信息==================");
                println!(
                    "聪明钱进行现货买入订单: 代币：{} 价格：{} 数量: {}",
                    trade.coin, trade.px, trade.sz
                );
                execute_spot_buy_order(&trade, exchange_client.clone()).await?;
                // 限价单 可以挂上止盈止损单
                // execute_spot_limit_sell_order(&trade, exchange_client.clone()).await?;
            }
            "Sell" => {
                println!("===============聪明现货卖出信息==================");
                println!(
                    "聪明钱进行现货卖出订单: 代币：{} 价格：{} 数量: {}",
                    trade.coin, trade.px, trade.sz
                );
                if enable_sell {
                    execute_spot_sell_order(&trade, exchange_client.clone(), query_client.clone())
                        .await?;
                }
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
    // 解析原始价格
    let original_price = trade.px.parse::<f64>().unwrap();
    // 计算加价10%后的价格
    let limit_price = original_price * 1.05;
    // 获取原始价格的小数位数
    let decimal_places = if trade.px.contains('.') {
        trade.px.split('.').last().unwrap().len() as u32
    } else {
        0
    };
    // 调整价格精度，保持与原始价格相同的小数位数
    let adjusted_price = (limit_price * 10f64.powi(decimal_places as i32)).round()
        / 10f64.powi(decimal_places as i32);
    println!("执行现货买入 {} {} 跟单", trade.coin, adjusted_price,);
    let order = ClientOrderRequest {
        asset: trade.coin.to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: adjusted_price,
        sz: (TRADE_AMOUNT_USDT / trade.px.parse::<f64>().unwrap()),
        cloid: None,
        // 立即成交
        // Alo 挂单  Ioc 立即成交  否则失败 Gtc 订单保持有效直到被取消或完全成交
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Ioc".to_string(),
        }),
    };

    let response = match exchange_client.order(order, None).await {
        Ok(resp) => resp,
        Err(e) => {
            println!("交易失败: 错误详情 - {}", e);
            bail!(e)
        }
    };
    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("oid status错误: {status:?}"),
    };
    println!("--限价单--跟单购买成功： 订单id {}", oid);
    Ok(())
}

// 立即成交
async fn execute_spot_sell_order(
    trade: &TradeInfo,
    exchange_client: Arc<ExchangeClient>,
    query_client: Arc<InfoClient>,
) -> Result<()> {
    println!("执行现货卖出 {} 跟单", trade.coin);
    let spot_meta_path = Path::new("info").join("spot-meta.json");
    let spot_meta_json = match fs::read_to_string(spot_meta_path) {
        Ok(content) => content,
        Err(e) => {
            println!("无法读取spot-meta.json: {}", e);
            bail!("无法读取资产信息文件");
        }
    };
    let spot_meta = match serde_json::from_str::<SpotMeta>(&spot_meta_json) {
        Ok(meta) => meta,
        Err(e) => {
            println!("解析spot-meta.json失败: {}", e);
            bail!("资产信息文件格式错误");
        }
    };
    let spot_universe_info = spot_meta
        .universe
        .iter()
        .find(|asset| asset.name == trade.coin)
        .ok_or_else(|| anyhow::anyhow!("未找到交易对 {}", trade.coin))?;

    let current_spot_token_info = spot_meta.tokens[&spot_universe_info.index + 1].clone();

    let my_all_token_balances = query_client
        .user_token_balances(H160::from_str(MY_ADDRESS).unwrap())
        .await?;
    let current_spot = my_all_token_balances
        .balances
        .iter()
        .find(|token| token.coin == current_spot_token_info.name)
        .unwrap();
    let current_spot_balance = &current_spot.total;

    println!(
        "当前代币 {:#?} 余额：{}",
        current_spot_token_info, current_spot_balance
    );
    // 解析原始价格
    let original_price = trade.px.parse::<f64>().unwrap();
    // 计算加价10%后的价格
    let limit_price = original_price * 0.95;
    // 获取原始价格的小数位数
    let decimal_places = if trade.px.contains('.') {
        trade.px.split('.').last().unwrap().len() as u32
    } else {
        0
    };
    // 调整价格精度，保持与原始价格相同的小数位数
    let adjusted_price = (limit_price * 10f64.powi(decimal_places as i32)).round()
        / 10f64.powi(decimal_places as i32);
    // 根据基础代币的szDecimals调整数量
    let adjusted_size = if current_spot_token_info.sz_decimals == 0 {
        current_spot_balance.parse::<f64>()?.floor()
    } else {
        let factor = 10f64.powi(current_spot_token_info.sz_decimals as i32);
        (current_spot_balance.parse::<f64>()? * factor).floor() / factor
    };
    println!("交易价格 {}  交易sz {}", adjusted_price, adjusted_size);
    let order = ClientOrderRequest {
        asset: trade.coin.to_string(),
        is_buy: false,
        reduce_only: false,
        limit_px: adjusted_price,
        sz: adjusted_size,
        cloid: None,
        // Alo 挂单  Ioc 立即成交  否则失败 Gtc 订单保持有效直到被取消或完全成交
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Ioc".to_string(),
        }),
    };
    let response = match exchange_client.order(order, None).await {
        Ok(resp) => resp,
        Err(e) => {
            println!("交易失败: 错误详情 - {}", e);
            bail!(e)
        }
    };
    println!("response:  {:#?}", response);
    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("oid status错误: {status:?}"),
    };
    println!("跟单卖出清仓： 订单id {}", oid);
    Ok(())
}
