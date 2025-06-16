use crate::{
    AppState,
    models::models::{AccountVerificationResponse, Bank, FlutterwaveBankApiResponse},
};
use actix_web::{HttpResponse, web};

pub async fn get_bank_code_and_verify_account(
    app_state: &web::Data<AppState>,
    bank_name: String,
    account_number: String,
) -> Result<(AccountVerificationResponse, String), HttpResponse> {
    let banks = match fetch_banks_via_flutterwave(&app_state).await {
        Ok(banks) => banks,
        Err(e) => {
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Failed to fetch banks",
                "data": null,
                "error": Some(e.to_string()),
            })));
        }
    };

    let bank_code = banks
        .iter()
        .find(|bank| bank.name == bank_name)
        .map(|bank| bank.code.clone());

    let bank_code = match bank_code {
        Some(code) => code,
        None => {
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Bank not found",
                "data": null,
                "error": Some(format!("Bank '{}' not found", bank_name)),
            })));
        }
    };

    match verify_account_via_flutterwave(&app_state, &bank_code, &account_number).await {
        Ok(account_details) => return Ok((account_details, bank_code)),
        Err(e) => {
            return Err(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Bank account verification failed",
                "data": null,
                "error": Some(e),
            })));
        }
    }
}

pub async fn fetch_banks_via_flutterwave(
    app_state: &web::Data<AppState>,
) -> Result<Vec<Bank>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.flutterwave.com/v3/banks/NG")
        .header(
            "Authorization",
            format!("Bearer {}", app_state.env.flutterwave_secret_key),
        )
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let raw_response = &response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    let banks_response: FlutterwaveBankApiResponse<Vec<Bank>> = serde_json::from_str(&raw_response)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    match banks_response.status.as_str() {
        "success" => Ok(banks_response.data),
        _ => Err(format!("API returned error: {}", banks_response.message)),
    }
}

pub async fn verify_account_via_flutterwave(
    app_state: &web::Data<AppState>,
    account_number: &str,
    bank_code: &str,
) -> Result<AccountVerificationResponse, String> {
    let client = reqwest::Client::new();

    let url = format!("https://api.flutterwave.com/v3/accounts/resolve");

    let payload = serde_json::json!({
        "account_number": account_number,
        "account_bank": bank_code
    });

    let response = client
        .post(&url)
        .header(
            "Authorization",
            format!("Bearer {}", app_state.env.flutterwave_secret_key),
        )
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    match response.status().is_success() {
        true => {
            let verification_response: FlutterwaveBankApiResponse<AccountVerificationResponse> =
                response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

            match verification_response.status.as_str() {
                "success" => Ok(verification_response.data),
                _ => Err(format!(
                    "Verification failed: {}",
                    verification_response.message
                )),
            }
        }
        false => {
            let status = response.status();
            let error_message = response.text().await.unwrap_or_default();
            Err(format!("API error: {} - {}", status, error_message))
        }
    }
}
