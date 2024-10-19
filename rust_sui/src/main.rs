use anyhow::anyhow;
use bcs;
use std::str::FromStr;
use std::time::Duration;
use sui_config::{sui_config_dir, SUI_CLIENT_CONFIG};
use sui_sdk::rpc_types::{ObjectChange, SuiObjectDataOptions};
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::{Argument, CallArg, Command, TransactionData};
use sui_sdk::types::Identifier;
use sui_sdk::wallet_context::WalletContext;
use sui_sdk::SuiClientBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Sui testnet -- https://fullnode.testnet.sui.io:443
    let sui_testnet = SuiClientBuilder::default().build_testnet().await?;
    println!("Sui testnet version: {}", sui_testnet.api_version());

    let active_address =
        SuiAddress::from_str("0x9e4092b6a894e6b168aa1c6c009f5c1c1fcb83fb95e5aa39144e1d2be4ee0d67")?;

    let total_balance = sui_testnet
        .coin_read_api()
        .get_all_balances(active_address)
        .await?;

    for balance in total_balance {
        println!("{}: {}", balance.coin_type, balance.total_balance);
    }

    let wallet_config = sui_config_dir()?.join(SUI_CLIENT_CONFIG);
    println!("{}", wallet_config.display());
    let mut wallet =
        WalletContext::new(&wallet_config, Some(Duration::from_secs(60)), None).unwrap();
    println!("{}", wallet.active_address().unwrap());

    let coins = sui_testnet
        .coin_read_api()
        .get_coins(active_address, None, None, None)
        .await?;
    let coin = coins.data.into_iter().next().unwrap(); // use as gas
    println!("{}: {}", coin.coin_type, coin.balance);

    let mut ptb = ProgrammableTransactionBuilder::new();

    let a = 333u64;
    let b = 666u64;
    let a_argument = CallArg::Pure(bcs::to_bytes(&a).unwrap());
    let b_argument = CallArg::Pure(bcs::to_bytes(&b).unwrap());
    ptb.input(a_argument)?;
    ptb.input(b_argument)?;

    let pkg_id = "0xe2496799139225a06e7251857cdf46a32c20d773030c62b9bf24095cd60aac43";
    let package = ObjectID::from_hex_literal(pkg_id).map_err(|e| anyhow!(e))?;
    let module = Identifier::new("my_module").map_err(|e| anyhow!(e))?;
    let function = Identifier::new("add").map_err(|e| anyhow!(e))?;
    ptb.command(Command::move_call(
        package,
        module,
        function,
        vec![],
        vec![Argument::Input(0), Argument::Input(1)],
    ));

    let builder = ptb.finish();

    let gas_budget = 10_000_000u64;
    let gas_price = sui_testnet.read_api().get_reference_gas_price().await?;
    let tx_data = TransactionData::new_programmable(
        active_address,
        vec![coin.object_ref()],
        builder,
        gas_budget,
        gas_price,
    );

    let tx = wallet.sign_transaction(&tx_data);
    let res = wallet.execute_transaction_must_succeed(tx).await;

    // println!("{}", res);

    for object in res.object_changes.unwrap() {
        match object {
            ObjectChange::Created {
                sender: _,
                owner: _,
                object_type,
                object_id,
                version,
                digest: _,
            } => {
                println!("{}: {}", object_type, object_id);
                let sui_data_options = SuiObjectDataOptions {
                    show_type: true,
                    show_owner: true,
                    show_previous_transaction: true,
                    show_display: true,
                    show_content: true,
                    show_bcs: true,
                    show_storage_rebate: true,
                };
                let object = sui_testnet.read_api().try_get_parsed_past_object(object_id, version, sui_data_options).await?;
                // println!("{:#?}", object);
                match object {
                    sui_sdk::rpc_types::SuiPastObjectResponse::VersionFound(data) => {
                        let output = data.content.unwrap().to_string().split(" ").last().unwrap().to_string();
                        println!("{a} + {b} = {output}");
                    },
                    _ => ()
                }
            }
            _ => continue,
        }
    }

    Ok(())
}
