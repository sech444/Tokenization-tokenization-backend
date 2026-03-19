// src/services/email.rs
// COMPLETE VERSION WITH BOTH PASSWORD RESET AND EMAIL VERIFICATION

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use crate::utils::errors::AppError;

pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
}

impl EmailService {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            smtp_host: std::env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .map_err(|_| AppError::ConfigurationError("Invalid SMTP_PORT".to_string()))?,
            smtp_username: std::env::var("SMTP_USERNAME")
                .map_err(|_| AppError::MissingConfiguration("SMTP_USERNAME not set".to_string()))?,
            smtp_password: std::env::var("SMTP_PASSWORD")
                .map_err(|_| AppError::MissingConfiguration("SMTP_PASSWORD not set".to_string()))?,
            from_email: std::env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@tokenization-platform.com".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "Tokenization Platform".to_string()),
        })
    }

    // ==================== PASSWORD RESET METHODS ====================

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        reset_link: &str,
    ) -> Result<(), AppError> {
        let html_body = self.generate_password_reset_html(to_name, reset_link);
        let text_body = self.generate_password_reset_text(to_name, reset_link);

        self.send_email(
            to_email,
            to_name,
            "Reset Your Password",
            &text_body,
            &html_body,
        )
        .await
    }

    fn generate_password_reset_html(&self, to_name: Option<&str>, reset_link: &str) -> String {
        let name = to_name.unwrap_or("there");
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Your Password</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background-color: #f4f7f9;
            margin: 0;
            padding: 0;
        }}
        .email-container {{
            max-width: 600px;
            margin: 40px auto;
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 40px 30px;
            text-align: center;
        }}
        .header h1 {{
            color: white;
            margin: 0;
            font-size: 28px;
            font-weight: 600;
        }}
        .content {{
            padding: 40px 30px;
        }}
        .button-container {{
            text-align: center;
            margin: 35px 0;
        }}
        .reset-button {{
            display: inline-block;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 16px 40px;
            text-decoration: none;
            border-radius: 6px;
            font-weight: 600;
            font-size: 16px;
        }}
        .footer {{
            background-color: #f8f9fa;
            padding: 30px;
            text-align: center;
            color: #6c757d;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="email-container">
        <div class="header">
            <h1>🔐 Password Reset Request</h1>
        </div>
        <div class="content">
            <p>Hi {},</p>
            <p>We received a request to reset your password. Click the button below to reset it:</p>
            <div class="button-container">
                <a href="{}" class="reset-button">Reset My Password</a>
            </div>
            <p>This link will expire in 1 hour. If you didn't request this, you can ignore this email.</p>
        </div>
        <div class="footer">
            <p><strong>Tokenization Platform</strong><br>support@tokenization-platform.com</p>
        </div>
    </div>
</body>
</html>
"#,
            name, reset_link
        )
    }

    fn generate_password_reset_text(&self, to_name: Option<&str>, reset_link: &str) -> String {
        let name = to_name.unwrap_or("there");
        format!(
            r#"
Password Reset Request

Hi {},

We received a request to reset your password for your Tokenization Platform account.

To reset your password, please visit: {}

This link will expire in 1 hour.

---
Tokenization Platform
support@tokenization-platform.com
"#,
            name, reset_link
        )
    }

    // ==================== EMAIL VERIFICATION METHODS ====================

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        verification_link: &str,
    ) -> Result<(), AppError> {
        let html_body = self.generate_verification_html(to_name, verification_link);
        let text_body = self.generate_verification_text(to_name, verification_link);

        self.send_email(
            to_email,
            to_name,
            "Verify Your Email Address",
            &text_body,
            &html_body,
        )
        .await
    }

    fn generate_verification_html(&self, to_name: Option<&str>, verification_link: &str) -> String {
        let name = to_name.unwrap_or("there");
        format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verify Your Email</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background-color: #f4f7f9;
            margin: 0;
            padding: 0;
        }}
        .email-container {{
            max-width: 600px;
            margin: 40px auto;
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 40px 30px;
            text-align: center;
        }}
        .header h1 {{
            color: white;
            margin: 0;
            font-size: 28px;
            font-weight: 600;
        }}
        .icon {{
            font-size: 48px;
            margin-bottom: 10px;
        }}
        .content {{
            padding: 40px 30px;
        }}
        .button-container {{
            text-align: center;
            margin: 35px 0;
        }}
        .verify-button {{
            display: inline-block;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 16px 40px;
            text-decoration: none;
            border-radius: 6px;
            font-weight: 600;
            font-size: 16px;
        }}
        .info-box {{
            background-color: #e7f3ff;
            border-left: 4px solid #2196F3;
            padding: 15px;
            margin: 25px 0;
            border-radius: 4px;
        }}
        .footer {{
            background-color: #f8f9fa;
            padding: 30px;
            text-align: center;
            color: #6c757d;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="email-container">
        <div class="header">
            <div class="icon">✉️</div>
            <h1>Welcome to Tokenization Platform!</h1>
        </div>
        <div class="content">
            <p>Hi {},</p>
            <p>Thank you for signing up! To complete your registration, please verify your email address.</p>
            <div class="button-container">
                <a href="{}" class="verify-button">Verify Email Address</a>
            </div>
            <div class="info-box">
                <p>ℹ️ This verification link will expire in 24 hours. If you didn't create an account, you can ignore this email.</p>
            </div>
        </div>
        <div class="footer">
            <p><strong>Tokenization Platform</strong><br>support@tokenization-platform.com</p>
        </div>
    </div>
</body>
</html>
"#,
            name, verification_link
        )
    }

    fn generate_verification_text(&self, to_name: Option<&str>, verification_link: &str) -> String {
        let name = to_name.unwrap_or("there");
        format!(
            r#"
Welcome to Tokenization Platform!

Hi {},

Thank you for signing up! To complete your registration, please verify your email address.

Verification link: {}

This link will expire in 24 hours.

---
Tokenization Platform
support@tokenization-platform.com
"#,
            name, verification_link
        )
    }

    // ==================== SHARED EMAIL SENDING METHOD ====================

    async fn send_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        subject: &str,
        _text_body: &str,
        html_body: &str,
    ) -> Result<(), AppError> {
        let from_mailbox = format!("{} <{}>", self.from_name, self.from_email)
            .parse::<Mailbox>()
            .map_err(|e| AppError::EmailServiceError(format!("Invalid from address: {}", e)))?;

        let to_mailbox = if let Some(name) = to_name {
            format!("{} <{}>", name, to_email)
        } else {
            to_email.to_string()
        }
        .parse::<Mailbox>()
        .map_err(|e| AppError::EmailServiceError(format!("Invalid to address: {}", e)))?;

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| AppError::EmailServiceError(format!("Failed to build email: {}", e)))?;

        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

        let mailer = SmtpTransport::starttls_relay(&self.smtp_host)
            .map_err(|e| AppError::EmailServiceError(format!("Failed to create SMTP transport: {}", e)))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| AppError::EmailServiceError(format!("Failed to send email: {}", e)))?;

        tracing::info!("Email sent successfully to: {}", to_email);
        Ok(())
    }
}