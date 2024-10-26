use anyhow::anyhow;
use std::time::Duration;
use std::{env, fs};
use sui_config::{sui_config_dir, SUI_CLIENT_CONFIG};
use sui_move_build::BuildConfig;
use sui_sdk::rpc_types::ObjectChange;
use sui_sdk::types::base_types::ObjectID;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::TransactionData;
use sui_sdk::types::Identifier;
use sui_sdk::wallet_context::WalletContext;
use sui_sdk::SuiClientBuilder;
use serde_json::Value;

pub async fn publish() -> Result<(), anyhow::Error> {
    let sui_testnet = SuiClientBuilder::default().build_testnet().await?;

    let wallet_config = sui_config_dir()?.join(SUI_CLIENT_CONFIG);
    let mut wallet =
        WalletContext::new(&wallet_config, Some(Duration::from_secs(60)), None).unwrap();
    let active_address = wallet.active_address().unwrap();

    let coins = sui_testnet
        .coin_read_api()
        .get_coins(active_address, None, None, None)
        .await?;
    // use to gas
    let coin = coins.data.into_iter().next().unwrap();

    let mut build_config = BuildConfig::default();
    build_config.print_diags_to_stderr = true;
    build_config.run_bytecode_verifier = true;

    let current_dir = env::current_dir()?;
    let source_code = current_dir.join("hello");
    let pkg = build_config.build(source_code.as_path()).unwrap();
    let modules = pkg.get_package_bytes(true);
    let ids = pkg.dependency_ids.published;
    let mut dep_ids = Vec::<ObjectID>::new();
    for (_name, address) in ids {
        dep_ids.push(address);
    }

    let mut ptb = ProgrammableTransactionBuilder::new();
    ptb.publish_immutable(modules, dep_ids);
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
        if let ObjectChange::Published {
            package_id,
            version: _,
            digest: _,
            modules: _,
        } = object
        {
            // println!("{package_id}");
            let coins = sui_testnet
                .coin_read_api()
                .get_coins(active_address, None, None, None)
                .await?;
            // use to gas
            let coin = coins.data.into_iter().next().unwrap();
            let mut ptb = ProgrammableTransactionBuilder::new();
            let module = Identifier::new("hello").map_err(|e| anyhow!(e))?;
            let function = module.clone();
            ptb.move_call(package_id, module, function, vec![], vec![])
                .unwrap();
            let builder = ptb.finish();
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

            if let Some(events) = res.events {
                for event in events.data {
                    // println!("{}", event.parsed_json);
                    if let Value::Object(mp) = event.parsed_json {
                        for key in mp.keys() {
                            println!("{}: {}", key, mp.get(key).unwrap());
                        }
                    }
                }
            }
        }
    }

    let build_file = source_code.join("build");
    fs::remove_dir_all(build_file.as_path()).unwrap();

    Ok(())
}
