use std::{env, fs, path::Path, str::FromStr, sync::Arc};

use ethers::{signers::LocalWallet, types::H160};
use hype_copy_trade::{
    constants::USER_ADDRESS,
    handler::{handle_trades_event::handle_trades_event, handle_user_event::handle_user_event},
};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, InfoClient, Message, Subscription};

use log::debug;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();
    let query_client: InfoClient = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();
    let query_client: Arc<InfoClient> = Arc::new(query_client);
    let user = H160::from_str(USER_ADDRESS).unwrap();

    let (sender, mut receiver) = unbounded_channel();

    let _ = info_client
        .subscribe(Subscription::UserEvents { user }, sender.clone())
        .await
        .unwrap();
    let wallet: LocalWallet = env::var("PRIVATE_KEY").unwrap().parse().unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();
    let exchange_client = Arc::new(exchange_client);
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
                    let exchange_client_clone = exchange_client.clone();
                    let trade_infos_clone = trade_infos.clone();
                    let query_client_clone = query_client.clone();
                    tokio::spawn(async {
                        let _ = handle_user_event(
                            trade_infos_clone,
                            exchange_client_clone,
                            query_client_clone,
                        )
                        .await;
                    });
                }
                _ => {}
            },
            Message::Pong => {
                debug!("pong");
            }
            _ => {}
        }
    }
}
