// Teste isolado para o módulo masker
// Este arquivo pode ser executado com: cargo test --test test_masker

// Definições temporárias para evitar dependências externas
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Masker {
    patterns: HashMap<String, Regex>,
}

impl Masker {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // CPF pattern - matches xxx.xxx.xxx-xx or xxxxxxxxxxx
        patterns.insert(
            "cpf".to_string(),
            Regex::new(r"\b(\d{3})\.?(\d{3})\.?(\d{3})-?(\d{2})\b").unwrap(),
        );
        
        // Email pattern
        patterns.insert(
            "email".to_string(),
            Regex::new(r"\b([a-zA-Z0-9._%+-]+)@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,})\b").unwrap(),
        );
        
        // Phone pattern - Brazilian format
        patterns.insert(
            "phone".to_string(),
            Regex::new(r"\b\(?\d{2}\)?\s*\d{4,5}-?\d{4}\b").unwrap(),
        );
        
        // Credit card pattern
        patterns.insert(
            "credit_card".to_string(),
            Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap(),
        );
        
        Self { patterns }
    }
    
    pub fn mask_text(&self, text: &str) -> String {
        let mut masked_text = text.to_string();
        
        for (name, regex) in &self.patterns {
            masked_text = match name.as_str() {
                "cpf" => {
                    regex.replace_all(&masked_text, |caps: &regex::Captures| {
                        format!("***.***.***-{}", &caps[4])
                    }).to_string()
                },
                "email" => {
                    regex.replace_all(&masked_text, |caps: &regex::Captures| {
                        let username = &caps[1];
                        let domain = &caps[2];
                        if username.len() > 1 {
                            format!("{}***@{}", &username[0..1], domain)
                        } else {
                            format!("***@{}", domain)
                        }
                    }).to_string()
                },
                "phone" => {
                    regex.replace_all(&masked_text, "(***) ***-****").to_string()
                },
                "credit_card" => {
                    regex.replace_all(&masked_text, "**** **** **** ****").to_string()
                },
                _ => masked_text,
            };
        }
        
        masked_text
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
        assert_eq!(masked, "Meu CPF é ***.***.***-01");
        
        // Test unformatted CPF
        let text = "CPF: 12345678901";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "CPF: ***.***.***-01");
    }

    #[test]
    fn test_email_masking() {
        let masker = Masker::new();
        
        let text = "Meu email é joao@exemplo.com";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "Meu email é j***@exemplo.com");
        
        // Test short email
        let text = "a@b.com";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "***@b.com");
    }

    #[test]
    fn test_phone_masking() {
        let masker = Masker::new();
        
        // Test formatted phone
        let text = "Ligue para (11) 99999-1234";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "Ligue para (***) ***-****");
        
        // Test unformatted phone
        let text = "Tel: 11999991234";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "Tel: (***) ***-****");
    }

    #[test]
    fn test_credit_card_masking() {
        let masker = Masker::new();
        
        let text = "Cartão: 1234 5678 9012 3456";
        let masked = masker.mask_text(text);
        assert_eq!(masked, "Cartão: **** **** **** ****");
    }

    #[test]
    fn test_multiple_pii_masking() {
        let masker = Masker::new();
        
        let text = "Cliente: João Silva, CPF: 123.456.789-00, Email: joao@example.com, Tel: (11) 98765-4321";
        let masked = masker.mask_text(text);
        
        assert!(masked.contains("***.***.***-00"));
        assert!(masked.contains("j***@example.com"));
        assert!(masked.contains("(***) ***-****"));
    }

    #[test]
    fn test_no_pii() {
        let masker = Masker::new();
        
        let text = "Este é um texto sem informações pessoais";
        let masked = masker.mask_text(text);
        assert_eq!(masked, text);
    }
} 