use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, warn};
use crate::agent::KeyEvent;

#[derive(Debug, Clone)]
pub struct Masker {
    patterns: HashMap<String, Regex>,
}

impl Masker {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // CPF pattern (000.000.000-00 or 00000000000)
        if let Ok(cpf_regex) = Regex::new(r"\b\d{3}\.?\d{3}\.?\d{3}-?\d{2}\b") {
            patterns.insert("cpf".to_string(), cpf_regex);
        }
        
        // Email pattern
        if let Ok(email_regex) = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b") {
            patterns.insert("email".to_string(), email_regex);
        }
        
        // Phone pattern (Brazilian format)
        if let Ok(phone_regex) = Regex::new(r"\b(?:\+55\s?)?\(?[1-9]{2}\)?\s?9?\d{4}-?\d{4}\b") {
            patterns.insert("phone".to_string(), phone_regex);
        }
        
        // Credit card pattern (basic)
        if let Ok(cc_regex) = Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b") {
            patterns.insert("credit_card".to_string(), cc_regex);
        }
        
        // RG pattern (Brazilian ID)
        if let Ok(rg_regex) = Regex::new(r"\b\d{1,2}\.?\d{3}\.?\d{3}-?[0-9X]\b") {
            patterns.insert("rg".to_string(), rg_regex);
        }
        
        // CNPJ pattern (Brazilian company ID)
        if let Ok(cnpj_regex) = Regex::new(r"\b\d{2}\.?\d{3}\.?\d{3}/?\d{4}-?\d{2}\b") {
            patterns.insert("cnpj".to_string(), cnpj_regex);
        }

        Self { patterns }
    }

    pub fn mask_event(&self, mut event: KeyEvent) -> KeyEvent {
        // Mascara o conteúdo da tecla
        event.key = self.mask_text(&event.key);
        
        // Mascara informações da janela se existirem
        if let Some(window_info) = &mut event.window_info {
            window_info.title = self.mask_text(&window_info.title);
            window_info.application = self.mask_text(&window_info.application);
        }
        
        event
    }

    pub fn mask_text(&self, text: &str) -> String {
        let mut masked_text = text.to_string();
        
        for (pattern_name, regex) in &self.patterns {
            if regex.is_match(&masked_text) {
                debug!("🔒 Mascarando padrão {} no texto", pattern_name);
                masked_text = regex.replace_all(&masked_text, |caps: &regex::Captures| {
                    self.generate_mask(&caps[0], pattern_name)
                }).to_string();
            }
        }
        
        masked_text
    }

    fn generate_mask(&self, original: &str, pattern_type: &str) -> String {
        match pattern_type {
            "cpf" => {
                if original.len() >= 11 {
                    format!("***.***.***-{}", &original[original.len()-2..])
                } else {
                    "***.***.**-**".to_string()
                }
            },
            "email" => {
                if let Some(at_pos) = original.find('@') {
                    let (local, domain) = original.split_at(at_pos);
                    if local.len() > 2 {
                        format!("{}***{}", &local[..1], &domain)
                    } else {
                        "***@***".to_string()
                    }
                } else {
                    "***@***".to_string()
                }
            },
            "phone" => {
                let digits_only: String = original.chars().filter(|c| c.is_digit(10)).collect();
                if digits_only.len() >= 8 {
                    format!("(***) ***-{}", &digits_only[digits_only.len()-4..])
                } else {
                    "(***) ***-****".to_string()
                }
            },
            "credit_card" => {
                let digits_only: String = original.chars().filter(|c| c.is_digit(10)).collect();
                if digits_only.len() >= 4 {
                    format!("**** **** **** {}", &digits_only[digits_only.len()-4..])
                } else {
                    "**** **** **** ****".to_string()
                }
            },
            "rg" => {
                "**.***.**-*".to_string()
            },
            "cnpj" => {
                "**.***.***/****-**".to_string()
            },
            _ => {
                warn!("⚠️ Tipo de padrão desconhecido: {}", pattern_type);
                "*".repeat(original.len())
            }
        }
    }

    pub fn add_custom_pattern(&mut self, name: String, pattern: String) -> Result<(), regex::Error> {
        let regex = Regex::new(&pattern)?;
        self.patterns.insert(name, regex);
        Ok(())
    }

    pub fn remove_pattern(&mut self, name: &str) -> bool {
        self.patterns.remove(name).is_some()
    }

    pub fn list_patterns(&self) -> Vec<String> {
        self.patterns.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpf_masking() {
        let masker = Masker::new();
        
        // Test formatted CPF
        let text = "Meu CPF é 123.456.789-01";
        let masked = masker.mask_text(text);
        assert!(masked.contains("***.***.***-01"));
        
        // Test unformatted CPF
        let text = "CPF: 12345678901";
        let masked = masker.mask_text(text);
        assert!(masked.contains("***.***.***-01"));
    }

    #[test]
    fn test_email_masking() {
        let masker = Masker::new();
        
        let text = "Meu email é joao@exemplo.com";
        let masked = masker.mask_text(text);
        assert!(masked.contains("j***@exemplo.com"));
        
        // Test short email
        let text = "a@b.com";
        let masked = masker.mask_text(text);
        assert!(masked.contains("a***@b.com"));
    }

    #[test]
    fn test_phone_masking() {
        let masker = Masker::new();
        
        let text = "Telefone: (11) 99999-1234";
        let masked = masker.mask_text(text);
        assert!(masked.contains("(***) ***-1234"));
        
        // Test phone with country code
        let text = "+55 11 99999-1234";
        let masked = masker.mask_text(text);
        assert!(masked.contains("(***) ***-1234"));
    }

    #[test]
    fn test_credit_card_masking() {
        let masker = Masker::new();
        
        // Test with spaces
        let text = "Card: 1234 5678 9012 3456";
        let masked = masker.mask_text(text);
        assert!(masked.contains("**** **** **** 3456"));
        
        // Test with dashes
        let text = "1234-5678-9012-3456";
        let masked = masker.mask_text(text);
        assert!(masked.contains("**** **** **** 3456"));
    }

    #[test]
    fn test_rg_masking() {
        let masker = Masker::new();
        
        let text = "RG: 12.345.678-9";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "RG: **.***.***-*");
    }

    #[test]
    fn test_cnpj_masking() {
        let masker = Masker::new();
        
        let text = "CNPJ: 12.345.678/0001-90";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "CNPJ: **.***.***/****-**");
    }

    #[test]
    fn test_multiple_patterns() {
        let masker = Masker::new();
        
        let text = "CPF: 123.456.789-01 Email: test@test.com Telefone: (11) 99999-1234";
        let masked = masker.mask_text(text);
        
        assert!(masked.contains("***.***.***-01"));
        assert!(masked.contains("t***@test.com"));
        assert!(masked.contains("(***) ***-1234"));
    }

    #[test]
    fn test_mask_event() {
        let masker = Masker::new();
        
        let event = KeyEvent {
            timestamp: 1234567890,
            key: "test@example.com".to_string(),
            event_type: "press".to_string(),
            window_info: Some("Email: test@example.com - Phone: (11) 99999-1234".to_string()),
            is_modifier: false,
            is_function_key: false,
        };
        
        let masked_event = masker.mask_event(event);
        
        assert_eq!(masked_event.key, "t***@example.com");
        assert_eq!(masked_event.window_info, Some("Email: t***@example.com - Phone: (***) ***-1234".to_string()));
    }

    #[test]
    fn test_custom_pattern() {
        let mut masker = Masker::new();
        
        // Add custom pattern for ID numbers
        masker.add_custom_pattern(
            "custom_id".to_string(), 
            r"\bID-\d{6}\b".to_string()
        ).unwrap();
        
        let text = "My ID is ID-123456";
        let masked = masker.mask_text(text);
        
        // Should use generic masking for unknown pattern types
        assert!(masked.contains("ID-"));
    }

    #[test]
    fn test_remove_pattern() {
        let mut masker = Masker::new();
        
        // Remove CPF pattern
        assert!(masker.remove_pattern("cpf"));
        
        // Try to remove non-existent pattern
        assert!(!masker.remove_pattern("non_existent"));
        
        // CPF should not be masked anymore
        let text = "CPF: 123.456.789-01";
        let masked = masker.mask_text(text);
        assert_eq!(masked, text);
    }

    #[test]
    fn test_list_patterns() {
        let masker = Masker::new();
        let patterns = masker.list_patterns();
        
        assert!(patterns.contains(&"cpf".to_string()));
        assert!(patterns.contains(&"email".to_string()));
        assert!(patterns.contains(&"phone".to_string()));
        assert!(patterns.contains(&"credit_card".to_string()));
        assert!(patterns.contains(&"rg".to_string()));
        assert!(patterns.contains(&"cnpj".to_string()));
    }

    #[test]
    fn test_edge_cases() {
        let masker = Masker::new();
        
        // Empty text
        assert_eq!(masker.mask_text(""), "");
        
        // Text with no patterns
        let text = "This is just regular text with no sensitive data";
        assert_eq!(masker.mask_text(text), text);
        
        // Multiple occurrences of same pattern
        let text = "Emails: john@test.com and jane@test.com";
        let masked = masker.mask_text(text);
        assert!(masked.contains("j***@test.com") && masked.contains("j***@test.com"));
    }

    #[test]
    fn test_generate_mask_edge_cases() {
        let masker = Masker::new();
        
        // Test short CPF
        assert_eq!(masker.generate_mask("123", "cpf"), "***.***.**-**");
        
        // Test email without @ symbol
        assert_eq!(masker.generate_mask("notanemail", "email"), "***@***");
        
        // Test short phone number
        assert_eq!(masker.generate_mask("1234", "phone"), "(***) ***-****");
    }
} 