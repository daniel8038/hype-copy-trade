use anyhow::Result;
use hyperliquid_rust_sdk::InfoClient;
use serde_json;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub async fn info_init(query_client_clone: Arc<InfoClient>) -> Result<()> {
    // 获取spot_meta数据
    let spot_meta = query_client_clone.spot_meta().await?;

    // 将数据序列化为JSON字符串
    let json_data = serde_json::to_string_pretty(&spot_meta)?;

    // 确保info目录存在
    let info_dir = Path::new("info");
    if !info_dir.exists() {
        fs::create_dir(info_dir)?;
    }

    // 写入文件
    let file_path = info_dir.join("spot-meta.json");
    fs::write(&file_path, json_data)?;

    println!("已将spot_meta信息保存到 {:?}", file_path);

    Ok(())
}
