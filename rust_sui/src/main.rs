use std::str::FromStr;
use sui_sdk::types::base_types::SuiAddress;
use sui_sdk::SuiClientBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Sui testnet -- https://fullnode.testnet.sui.io:443
    let sui_testnet = SuiClientBuilder::default().build_testnet().await?;
    println!("Sui testnet version: {}", sui_testnet.api_version());

    let active_address = SuiAddress::from_str("0x9e4092b6a894e6b168aa1c6c009f5c1c1fcb83fb95e5aa39144e1d2be4ee0d67")?;

    let total_balance = sui_testnet
        .coin_read_api()
        .get_all_balances(active_address)
        .await?;

    for balance in total_balance {
        println!("{}: {}", balance.coin_type, balance.total_balance);
    }

    Ok(())
}