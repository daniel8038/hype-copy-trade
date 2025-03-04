use std::{fs, path::Path, str::FromStr};

use ethers::types::H160;
use hype_copy_trade::{
    constants::USER_ADDRESS,
    handler::{handle_trades_event::handle_trades_event, handle_user_event::handle_user_event},
};
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};

use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();
    let user = H160::from_str(USER_ADDRESS).unwrap();

    let (sender, mut receiver) = unbounded_channel();

    let _ = info_client
        .subscribe(Subscription::UserEvents { user }, sender.clone())
        .await
        .unwrap();
    // 定时获取 现货数据
    // tokio::spawn(async move {
    //     let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));

    //     loop {
    //         interval.tick().await;

    //         println!("定时获取现货数据...");
    //         match info_client.spot_meta().await {
    //             Ok(spot_meta) => {
    //                 // 确保目录存在
    //                 if let Err(e) = fs::create_dir_all("info") {
    //                     println!("创建目录失败: {}", e);
    //                     continue;
    //                 }

    //                 // 写入文件
    //                 if let Err(e) = fs::write(Path::new("info/spot_meta.json"), &spot_meta) {
    //                     println!("写入文件失败: {}", e);
    //                 } else {
    //                     println!("现货数据已保存到 info/spot_meta.json");
    //                 }
    //             }
    //             Err(e) => {
    //                 println!("获取现货数据失败: {}", e);
    //             }
    //         }
    //     }
    // });
    // let _ = info_client
    //     .subscribe(
    //         Subscription::Trades {
    //             coin: ("@107".into()),
    //         },
    //         sender.clone(),
    //     )
    //     .await
    //     .unwrap();
    // this loop ends when we unsubscribe
    while let Some(message) = receiver.recv().await {
        match message {
            // Message::AllMids(all_mids) => {}
            // Message::L2Book(l2_book) => {}
            // Message::UserFills(user_fills) => {}
            // Message::Candle(candle) => {}
            // Message::SubscriptionResponse => {}
            // Message::OrderUpdates(order_updates) => {}
            // Message::UserFundings(user_fundings) => {}
            // Message::UserNonFundingLedgerUpdates(user_non_funding_ledger_updates) => {}
            // Message::Notification(notification) => {}
            // Message::HyperliquidError(_) => {}
            // Message::WebData2(web_data2) => {}
            // Message::ActiveAssetCtx(active_asset_ctx) => {}
            // Message::SubscriptionResponse => {}
            // Message::NoData => {}
            //
            Message::Trades(_) => {
                // 这是关于token的全量交易数据
                // let trades_clone = trades.data.clone();
                // println!("{:#?}", trades_clone)
            }
            // 合约
            Message::User(user) => match user.data {
                // hyperliquid_rust_sdk::UserData::Funding(user_funding) => todo!(),
                // hyperliquid_rust_sdk::UserData::Liquidation(liquidation) => todo!(),
                // hyperliquid_rust_sdk::UserData::NonUserCancel(non_user_cancels) => todo!(),
                hyperliquid_rust_sdk::UserData::Fills(trade_infos) => {
                    let trade_infos_clone = trade_infos.clone();
                    tokio::spawn(async move {
                        let _ = handle_user_event(trade_infos_clone).await;
                    });
                }
                _ => {}
            },
            Message::Pong => {
                println!("收到pong");
            }
            _ => {}
        }
    }
}
