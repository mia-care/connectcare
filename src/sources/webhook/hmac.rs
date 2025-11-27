use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use crate::error::{AppError, Result};

type HmacSha256 = Hmac<Sha256>;

pub struct HmacValidator {
    secret: String,
    header_name: String,
}

impl HmacValidator {
    pub fn new(secret: String, header_name: String) -> Self {
        Self { secret, header_name }
    }
    
    pub fn header_name(&self) -> &str {
        &self.header_name
    }
    
    pub fn validate(&self, body: &[u8], signature_header: &str) -> Result<()> {
        tracing::debug!("Validating HMAC signature. Header: {}", signature_header);
        
        let signature = signature_header
            .strip_prefix("sha256=")
            .ok_or_else(|| {
                tracing::error!("Invalid signature format. Expected 'sha256=<hex>', got: {}", signature_header);
                AppError::InvalidSignatureFormat
            })?;
        
        tracing::debug!("Extracted signature: {}", signature);
        self.validate_body(body, signature)
    }
    
    fn validate_body(&self, body: &[u8], expected_signature: &str) -> Result<()> {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|_| AppError::Config("Invalid HMAC secret".to_string()))?;
        
        mac.update(body);
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        let computed_signature = hex::encode(code_bytes);
        
        tracing::debug!("Computed signature: {}", computed_signature);
        tracing::debug!("Expected signature: {}", expected_signature);
        tracing::debug!("Body length: {} bytes", body.len());
        
        let matches = computed_signature.as_bytes()
            .ct_eq(expected_signature.as_bytes())
            .into();
        
        if matches {
            tracing::debug!("Signature validation successful");
            Ok(())
        } else {
            tracing::error!("Signature mismatch! Computed: {}, Expected: {}", computed_signature, expected_signature);
            Err(AppError::HmacValidation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hmac_validation_success() {
        let secret = "test_secret";
        let body = b"test body content";
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let expected = hex::encode(mac.finalize().into_bytes());
        
        let validator = HmacValidator::new(secret.to_string(), "X-Hub-Signature".to_string());
        let signature_header = format!("sha256={}", expected);
        
        assert!(validator.validate(body, &signature_header).is_ok());
    }
    
    #[test]
    fn test_hmac_validation_failure() {
        let validator = HmacValidator::new("test_secret".to_string(), "X-Hub-Signature".to_string());
        let result = validator.validate(b"test body", "sha256=wrongsignature");
        assert!(result.is_err());
    }
}
