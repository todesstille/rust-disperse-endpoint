use std::sync::Arc;
use dotenv::dotenv;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use ethers::{
    core::types::TransactionRequest, middleware::SignerMiddleware, prelude::abigen, providers::{Http, Middleware, Provider}, signers::{LocalWallet, Signer}, types::{Address, U256}
};

#[derive(Deserialize, Clone, Debug)]
pub struct CollectRequest {
    token: String,
    private_keys: Vec<String>,
    amounts: Vec<String>,
    destination: String
}

#[derive(Serialize)]
struct CollectResponse {
    token: String,
}

pub async fn make_collect(collect_req: web::Json<CollectRequest>) -> impl Responder {
    let collect_data = collect_req.into_inner();

    let mut handles = vec![];

    for i in 0..collect_data.private_keys.len() {
        let private_key = collect_data.private_keys[i].clone();
        let provider;

        match Provider::<Http>::try_from("https://sepolia.infura.io/v3/47bcfcab54cd4104a97fb13f84ae431e") {
            Ok(provider_inner) => {provider = provider_inner}
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to get provider: {}", err));
            }
        };

        let wallet;
        match LocalWallet::try_from(private_key) {
            Ok(wallet_inner) => {wallet = wallet_inner}
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to create wallet: {}", err));
            }
        }

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(11155111u64));

        let token_address;
        match collect_data.token.as_str().parse::<Address>() {
            Ok(address_inn) => {token_address = address_inn}
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to parse address: {}", err));
            }
        }

        let destination;
        match collect_data.destination.parse::<Address>() {
            Ok(destination_inner) => {destination = destination_inner}
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to parse amount: {}", err));
            }
        }

        let amount;
        match U256::from_dec_str(collect_data.amounts[i].as_str()) {
            Ok(amount_inner) => {amount = amount_inner}
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("Failed to parse amount: {}", err));
            }
        }


        handles.push(tokio::spawn(async move {
            if !token_address.is_zero() {
                abigen!(IERC20, "./token-abi.json");
                let token_contract = IERC20::new(token_address, Arc::new(client));

                if let Ok(tx) = token_contract.transfer(destination, amount).send().await {
                    let tx_hash = hex::encode(tx.tx_hash().as_bytes());
                    tx.await.expect("tx dropped from mempool");
                    return tx_hash;
                };
    
                return "".to_string();
            } else {
                if let Ok(tx) = client.send_transaction(
                    TransactionRequest::new()
                        .to(destination)
                        .value(amount), 
                    None).await {
                        let tx_hash = hex::encode(tx.tx_hash().as_bytes());
                        tx.await.expect("tx dropped from mempool");
                        return tx_hash;
                    };
        
                    return "".to_string();
                }
        }));
    }

    let mut txs = vec![];

    for handle in handles {
        match handle.await {
            Ok(tx) => {txs.push(tx)}
            Err(err) => {
                return HttpResponse::InternalServerError().body(format!("Failed to send transaction: {}", err));
            }

        }
    }
    
    for tx in txs.clone() {
        if tx == "" {
            return HttpResponse::InternalServerError().body(format!("Failed to send transaction"));
        }
    }
    
    return HttpResponse::Ok().json(txs);
}