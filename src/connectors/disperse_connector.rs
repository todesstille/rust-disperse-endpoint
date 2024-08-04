use actix_web::{web, HttpResponse, Responder};
use eyre::Context;
use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use ethers::{
    core::types::TransactionRequest, middleware::SignerMiddleware, providers::{Http, Middleware, Provider}, signers::{LocalWallet, Signer}, types::{Address, U256}
};

#[derive(Deserialize, Clone)]
pub struct DisperserRequest {
    token: String,
    addresses: Vec<String>,
    amounts: Vec<String>
}

#[derive(Serialize)]
struct DisperseResponse {
    token: String,
}

pub async fn make_disperse(disperser_req: web::Json<Vec<DisperserRequest>>) -> impl Responder {
    let disperser_data = disperser_req.into_inner();

    let mut calldata: Vec<u8> = vec![];
    calldata.push(disperser_data.len() as u8);
    for data in disperser_data {
        match data.get_calldata() {
            Ok(mut partial_calldata) => {
                calldata.append(&mut partial_calldata);
            },
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to parse data: {}", err));
            }
        }
    }

    let contract_address;
    match "0x88b81e18eC50eB04A83B82158DcC6dD1813ab6d0".parse::<Address>() {
        Ok(address_inn) => {contract_address = address_inn}
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Failed to parse address: {}", err));
        }
    }

    let provider;
    match Provider::<Http>::try_from("https://sepolia.infura.io/v3/47bcfcab54cd4104a97fb13f84ae431e") {
        Ok(provider_inner) => {provider = provider_inner}
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Failed to get provider: {}", err));
        }
    };

    let private_key;
    dotenv().ok();
    match env::var("PRIVATE_KEY") {
        Ok(value) => private_key = value,
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Failed to read disperser owner: {}", err));
        }
    }

    let wallet;
    match LocalWallet::try_from(private_key) {
        Ok(wallet_inner) => {wallet = wallet_inner}
        Err(err) => {
            return HttpResponse::BadRequest().body(format!("Failed to create wallet: {}", err));
        }
    }

    let client = SignerMiddleware::new(provider, wallet.with_chain_id(11155111u64));

    let tx;
    match client.send_transaction(
        TransactionRequest::new()
            .to(contract_address)
            .data(calldata),
        None).await {
            Ok(tx_inner) => {tx = tx_inner}
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to send tx: {}", err));
            }
        }

    let tx_hash = hex::encode(tx.tx_hash().as_bytes());
    tx.await.expect("tx dropped from mempool");
    return HttpResponse::Ok().json(tx_hash);
}

impl DisperserRequest {
    fn get_calldata(&self) -> eyre::Result<Vec<u8>> {
        let mut calldata: Vec<u8> = vec![];
        let mut token_address = hex::decode(&self.token[2..]).wrap_err("Cant deserialize token address")?;
        calldata.append(&mut token_address);
        calldata.push(self.addresses.len() as u8);

        for i in 0..self.addresses.len() {
            let mut destination = hex::decode(&self.addresses[i][2..]).wrap_err("Cant deserialize destination")?;
            calldata.append(&mut destination);

            let mut number = self.amounts[i].parse::<u128>().wrap_err("Cant parse amount")?;
            let mut number_bytes: Vec<u8> = vec![0; 32];
            for j in 0..32 {
                number_bytes[31 - j] = (number % 8) as u8;
                number /= 8;
            }
            
            calldata.append(&mut number_bytes);
        }

        Ok(calldata)
    }
}