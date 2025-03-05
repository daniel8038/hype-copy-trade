use std::{env, str::FromStr, sync::Arc, time::Duration};

use ethers::{signers::LocalWallet, types::H160};
use hype_copy_trade::{handler::handle_user_event::handle_user_event, utils::info_init};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, InfoClient, Message, Subscription};

use dotenv::dotenv;
use log::debug;
use tokio::sync::mpsc::unbounded_channel;
#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let is_test = env::var("TEST").unwrap().parse::<bool>().unwrap();
    let enable_sell = env::var("ENABLE_SELL").unwrap().parse::<bool>().unwrap();
    let enable_buy = env::var("ENABLE_BUY").unwrap().parse::<bool>().unwrap();
    let smart_address = env::var("SMART_ADDRESS").unwrap();
    println!(
        "是否为测试环境: {} 是否跟卖: {} 是否跟买: {}",
        is_test, enable_sell, enable_buy
    );
    let network = match is_test {
        true => BaseUrl::Testnet,
        false => BaseUrl::Mainnet,
    };
    let mut info_client = InfoClient::new(None, Some(network)).await.unwrap();
    let query_client: InfoClient = InfoClient::new(None, Some(network)).await.unwrap();
    let query_client: Arc<InfoClient> = Arc::new(query_client);
    let user = H160::from_str(&smart_address).unwrap();

    let (sender, mut receiver) = unbounded_channel();

    let _ = info_client
        .subscribe(Subscription::UserEvents { user }, sender.clone())
        .await
        .unwrap();
    let wallet: LocalWallet = env::var("PRIVATE_KEY").unwrap().parse().unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(network), None, None)
        .await
        .unwrap();
    let exchange_client = Arc::new(exchange_client);

    // 更新Info数据
    let query_info_client = query_client.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = info_init(query_info_client.clone()).await {
                eprintln!("更新spot_meta失败: {}", e);
            } else {
            }
            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    });

    // this loop ends when we unsubscribe
    while let Some(message) = receiver.recv().await {
        match message {
            Message::User(user) => match user.data {
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
