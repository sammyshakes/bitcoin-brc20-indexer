use super::{brc20_ticker::Brc20Ticker, Brc20Inscription};
use bitcoin::{Address, Network, TxIn};
use bitcoincore_rpc::{bitcoincore_rpc_json::GetRawTransactionResult, Client, RpcApi};
use log::error;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{DirBuilder, File},
    io::Write,
};

pub fn get_witness_data_from_raw_tx(
    raw_tx_info: &GetRawTransactionResult,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let transaction = raw_tx_info.transaction()?;

    let mut witness_data_strings: Vec<String> = Vec::new();

    // Get the first transaction input
    if let Some(input) = transaction.input.first() {
        // Iterate through each witness of the input
        for witness in &input.witness {
            let witness_string = String::from_utf8_lossy(witness).into_owned();
            witness_data_strings.push(witness_string);
        }
    }

    Ok(witness_data_strings)
}

// extracts only inscriptions that read "brc-20", many will be invalid
pub fn extract_and_process_witness_data(witness_data: String) -> Option<Brc20Inscription> {
    // Check for the correct MIME type and find its end
    let mime_end_index = if witness_data.contains("text/plain") {
        witness_data.find("text/plain").unwrap() + "text/plain".len()
    } else if witness_data.contains("application/json") {
        witness_data.find("application/json").unwrap() + "application/json".len()
    } else {
        return None;
    };

    // Start searching for the JSON data only after the MIME type
    if let Some(json_start) = witness_data[mime_end_index..].find('{') {
        let json_start = mime_end_index + json_start; // Adjust json_start to be relative to the original string
        if let Some(json_end) = witness_data[json_start..].rfind('}') {
            // Extract the JSON string
            let json_data = &witness_data[json_start..json_start + json_end + 1];

            // Try to parse the JSON data
            match serde_json::from_str::<Brc20Inscription>(json_data) {
                Ok(parsed_data) => {
                    // Only return the parsed data if it contains the expected fields
                    if parsed_data.p == "brc-20" {
                        // // Convert the data to JSON string with null values represented as "null"
                        // let json_string = serde_json::to_string(&parsed_data).unwrap_or_default();
                        // println!("{}", json_string);

                        return Some(parsed_data);
                    }
                }
                Err(_e) => {
                    // error!("JSON parsing failed: {:?}", e);
                }
            }
        }
    }

    None
}

pub fn get_owner_of_vout(
    raw_tx_info: &GetRawTransactionResult,
    vout_index: usize,
) -> Result<Address, anyhow::Error> {
    if raw_tx_info.vout.is_empty() {
        return Err(anyhow::anyhow!("Transaction has no outputs"));
    }

    if raw_tx_info.vout.len() <= vout_index {
        return Err(anyhow::anyhow!(
            "Transaction doesn't have vout at given index"
        ));
    }

    // Get the controlling address of vout[vout_index]
    let script_pubkey = &raw_tx_info.vout[vout_index].script_pub_key;
    let script = match script_pubkey.script() {
        Ok(script) => script,
        Err(e) => return Err(anyhow::anyhow!("Failed to get script: {:?}", e)),
    };
    let this_address = Address::from_script(&script, Network::Bitcoin).map_err(|e| {
        error!("Couldn't derive address from scriptPubKey: {:?}", e);
        anyhow::anyhow!("Couldn't derive address from scriptPubKey: {:?}", e)
    })?;

    Ok(this_address)
}

pub fn convert_to_float(number_string: &str, decimals: u8) -> Result<f64, &'static str> {
    let parts: Vec<&str> = number_string.split('.').collect();
    match parts.len() {
        1 => {
            // No decimal point in the string
            let result = number_string.parse::<f64>();
            match result {
                Ok(value) => Ok(value),
                Err(_) => Err("Malformed inscription"),
            }
        }
        2 => {
            // There is a decimal point in the string
            if parts[1].len() > decimals as usize {
                return Err("There are too many digits to the right of the decimal");
            } else {
                let result = number_string.parse::<f64>();
                match result {
                    Ok(value) => Ok(value),
                    Err(_) => Err("Malformed inscription"),
                }
            }
        }
        _ => Err("Malformed inscription"), // More than one decimal point
    }
}

pub fn transaction_inputs_to_values(client: &Client, inputs: &[TxIn]) -> anyhow::Result<Vec<u64>> {
    let mut values: Vec<u64> = vec![];

    for input in inputs {
        let prev_output = input.previous_output;
        println!(
            "Input from transaction: {:?}, index: {:?}",
            prev_output.txid, prev_output.vout
        );

        let prev_tx_info = client.get_raw_transaction_info(&prev_output.txid, None)?;

        let prev_tx = prev_tx_info.transaction()?;

        let output = &prev_tx.output[usize::try_from(prev_output.vout).unwrap()];

        // Add both the address and the value of the output to the list
        values.push(output.value);

        println!("=====");
    }

    if values.is_empty() {
        return Err(anyhow::anyhow!("Couldn't derive any values from inputs"));
    } else {
        Ok(values)
    }
}

//this is for logging to file
#[derive(Serialize)]
struct BalanceInfo {
    overall_balance: f64,
    available_balance: f64,
    transferable_balance: f64,
}

#[derive(Serialize)]
struct TickerWithBalances {
    ticker: Brc20Ticker,
    balances: HashMap<String, BalanceInfo>,
}

pub fn write_tickers_to_file(
    tickers: &HashMap<String, Brc20Ticker>,
    directory: &str,
) -> std::io::Result<()> {
    let mut dir_builder = DirBuilder::new();
    dir_builder.recursive(true); // This will create parent directories if they don't exist
    dir_builder.create(directory)?; // Create the directory if it doesn't exist

    for (ticker_name, ticker) in tickers {
        let filename = format!("{}/{}.json", directory, ticker_name); // create a unique filename
        let mut file = File::create(&filename)?; // create a new file for each ticker

        // map each balance to a BalanceInfo
        let balances: HashMap<String, BalanceInfo> = ticker
            .get_balances()
            .iter()
            .map(|(address, user_balance)| {
                (
                    address.to_string(),
                    BalanceInfo {
                        overall_balance: user_balance.get_overall_balance(),
                        available_balance: user_balance.get_available_balance(),
                        transferable_balance: user_balance.get_transferable_balance(),
                    },
                )
            })
            .collect();

        // construct a TickerWithBalances
        let ticker_with_balances = TickerWithBalances {
            ticker: ticker.clone(),
            balances,
        };

        // serialize and write the TickerWithBalances
        let serialized = serde_json::to_string_pretty(&ticker_with_balances)?;
        writeln!(file, "{}", serialized)?;
    }

    Ok(())
}
