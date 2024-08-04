use actix_web::{web, HttpResponse, Responder};
use eyre::Context;
use serde::{Deserialize, Serialize};

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

    // let response: Vec<DisperseResponse> = disperser_data.clone()
    //     .into_iter()
    //     .map(|req| DisperseResponse {token: req.token})
    //     .collect();

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

    return HttpResponse::Ok().json(hex::encode(calldata));
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
