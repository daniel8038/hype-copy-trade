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

    Ok(())
}
/// 根据原始价格和调整因子计算新价格，保持原始价格的小数精度
///
/// # 参数
/// * `original_price_str` - 原始价格，字符串格式
/// * `adjustment_factor` - 调整因子，例如1.05表示上调5%，0.95表示下调5%
///
/// # 返回值
/// * 调整后的价格，保持与原始价格相同的小数位数
///
/// # 示例
/// ```
/// let adjusted = format_adjust_price("123.45", 1.05);
/// assert_eq!(adjusted, 129.62);
/// ```
pub fn format_adjust_price(original_price_str: &str, adjustment_factor: f64) -> f64 {
    // 解析原始价格
    let original_price = original_price_str.parse::<f64>().unwrap();

    // 计算调整后的价格
    let adjusted_price_raw = original_price * adjustment_factor;

    // 获取原始价格的小数位数
    let decimal_places = if original_price_str.contains('.') {
        original_price_str.split('.').last().unwrap().len() as u32
    } else {
        0
    };

    // 调整价格精度，保持与原始价格相同的小数位数
    let adjusted_price = (adjusted_price_raw * 10f64.powi(decimal_places as i32)).round()
        / 10f64.powi(decimal_places as i32);

    adjusted_price
}
