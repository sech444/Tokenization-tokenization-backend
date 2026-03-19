// src/services/whatsapp.rs

use reqwest;
use serde::{Deserialize, Serialize};
use crate::utils::errors::AppError;

pub struct WhatsAppService {
    account_sid: String,
    auth_token: String,
    from_number: String, // Your WhatsApp Business number (e.g., "whatsapp:+14155238886")
}

impl WhatsAppService {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            account_sid: std::env::var("TWILIO_ACCOUNT_SID")
                .map_err(|_| AppError::MissingConfiguration("TWILIO_ACCOUNT_SID not set".to_string()))?,
            auth_token: std::env::var("TWILIO_AUTH_TOKEN")
                .map_err(|_| AppError::MissingConfiguration("TWILIO_AUTH_TOKEN not set".to_string()))?,
            from_number: std::env::var("TWILIO_WHATSAPP_NUMBER")
                .unwrap_or_else(|_| "whatsapp:+14155238886".to_string()), // Twilio sandbox default
        })
    }

    /// Generate a 6-digit OTP code
    pub fn generate_otp() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(100000..999999))
    }

    /// Send OTP via WhatsApp
    pub async fn send_otp(
        &self,
        to_phone: &str,
        otp: &str,
        user_name: Option<&str>,
    ) -> Result<(), AppError> {
        let message = self.format_otp_message(otp, user_name);
        self.send_whatsapp_message(to_phone, &message).await
    }

    /// Send verification success message
    pub async fn send_verification_success(
        &self,
        to_phone: &str,
        user_name: Option<&str>,
    ) -> Result<(), AppError> {
        let name = user_name.unwrap_or("there");
        let message = format!(
            "✅ *Phone Verified!*\n\nHi {},\n\nYour phone number has been successfully verified!\n\nYou can now access all features of the Tokenization Platform.\n\n_Tokenization Platform_",
            name
        );
        self.send_whatsapp_message(to_phone, &message).await
    }

    /// Send WhatsApp message via Twilio API
    async fn send_whatsapp_message(
        &self,
        to_phone: &str,
        message: &str,
    ) -> Result<(), AppError> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        // Ensure phone number has whatsapp: prefix
        let to_number = if to_phone.starts_with("whatsapp:") {
            to_phone.to_string()
        } else {
            format!("whatsapp:{}", to_phone)
        };

        let client = reqwest::Client::new();
        let params = [
            ("From", self.from_number.as_str()),
            ("To", to_number.as_str()),
            ("Body", message),
        ];

        let response = client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to send WhatsApp message: {}", e)))?;

        if response.status().is_success() {
            tracing::info!("WhatsApp OTP sent successfully to: {}", to_phone);
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Twilio API error: {}", error_text);
            Err(AppError::ExternalServiceError(format!(
                "Failed to send WhatsApp message: {}",
                error_text
            )))
        }
    }

    /// Format OTP message
    fn format_otp_message(&self, otp: &str, user_name: Option<&str>) -> String {
        let name = user_name.unwrap_or("there");
        format!(
            "🔐 *Verification Code*\n\nHi {},\n\nYour verification code is:\n\n*{}*\n\nThis code will expire in 10 minutes.\n\nIf you didn't request this code, please ignore this message.\n\n_Tokenization Platform_",
            name, otp
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwilioResponse {
    pub sid: String,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}