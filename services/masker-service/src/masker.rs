use crate::error::MaskingError;
use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingPattern {
    pub name: String,
    pub regex: String,
    pub replacement: String,
    pub enabled: bool,
    pub priority: u8, // Higher number = higher priority
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaskingResult {
    pub masked_text: String,
    pub detected_patterns: Vec<String>,
    pub pattern_counts: HashMap<String, usize>,
}

pub struct MaskingEngine {
    patterns: Vec<CompiledPattern>,
    keyword_matcher: Option<AhoCorasick>,
}

struct CompiledPattern {
    name: String,
    regex: Regex,
    replacement: String,
    enabled: bool,
    priority: u8,
    category: String,
}

// Default PII patterns with Brazilian focus
static DEFAULT_PATTERNS: Lazy<Vec<MaskingPattern>> = Lazy::new(|| {
    vec![
        // Brazilian CPF
        MaskingPattern {
            name: "cpf".to_string(),
            regex: r"\b\d{3}\.?\d{3}\.?\d{3}-?\d{2}\b".to_string(),
            replacement: "***.***.***-**".to_string(),
            enabled: true,
            priority: 10,
            category: "document".to_string(),
        },
        // Brazilian CNPJ
        MaskingPattern {
            name: "cnpj".to_string(),
            regex: r"\b\d{2}\.?\d{3}\.?\d{3}/?\d{4}-?\d{2}\b".to_string(),
            replacement: "**.***.***/****-**".to_string(),
            enabled: true,
            priority: 10,
            category: "document".to_string(),
        },
        // Email addresses
        MaskingPattern {
            name: "email".to_string(),
            regex: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            replacement: "***@***.***".to_string(),
            enabled: true,
            priority: 9,
            category: "contact".to_string(),
        },
        // Brazilian phone numbers
        MaskingPattern {
            name: "phone_br".to_string(),
            regex: r"(\+55\s?)?(\(?\d{2}\)?\s?)?\d{4,5}-?\d{4}".to_string(),
            replacement: "(**) *****-****".to_string(),
            enabled: true,
            priority: 8,
            category: "contact".to_string(),
        },
        // Credit card numbers
        MaskingPattern {
            name: "credit_card".to_string(),
            regex: r"\b(?:\d{4}[\s-]?){3}\d{4}\b".to_string(),
            replacement: "****-****-****-****".to_string(),
            enabled: true,
            priority: 10,
            category: "financial".to_string(),
        },
        // Brazilian bank account
        MaskingPattern {
            name: "bank_account".to_string(),
            regex: r"\b\d{4,6}-?\d{1,2}\b".to_string(),
            replacement: "******-*".to_string(),
            enabled: true,
            priority: 7,
            category: "financial".to_string(),
        },
        // IP addresses
        MaskingPattern {
            name: "ipv4".to_string(),
            regex: r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b".to_string(),
            replacement: "***.***.***.***".to_string(),
            enabled: true,
            priority: 6,
            category: "network".to_string(),
        },
        // Brazilian RG (Identity document)
        MaskingPattern {
            name: "rg".to_string(),
            regex: r"\b\d{1,2}\.?\d{3}\.?\d{3}-?\d{1}\b".to_string(),
            replacement: "**.***.**-*".to_string(),
            enabled: true,
            priority: 9,
            category: "document".to_string(),
        },
        // URLs with sensitive parameters
        MaskingPattern {
            name: "sensitive_url".to_string(),
            regex: r"https?://[^\s]+(?:token|key|password|secret)=[^\s&]+".to_string(),
            replacement: "https://***?***=***".to_string(),
            enabled: true,
            priority: 8,
            category: "security".to_string(),
        },
        // API keys (generic pattern)
        MaskingPattern {
            name: "api_key".to_string(),
            regex: r"\b[A-Za-z0-9]{20,64}\b".to_string(),
            replacement: "****API_KEY****".to_string(),
            enabled: false, // Disabled by default to avoid false positives
            priority: 5,
            category: "security".to_string(),
        },
    ]
});

// Sensitive keywords that might indicate passwords or secrets
static SENSITIVE_KEYWORDS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "password", "senha", "secret", "segredo", "token", "key", "chave",
        "auth", "login", "pass", "pwd", "credential", "credencial",
    ]
});

impl MaskingEngine {
    pub fn new() -> Self {
        let patterns = Self::compile_patterns(&DEFAULT_PATTERNS);
        let keyword_matcher = Self::build_keyword_matcher();
        
        Self {
            patterns,
            keyword_matcher,
        }
    }

    pub fn with_custom_patterns(patterns: Vec<MaskingPattern>) -> Result<Self, MaskingError> {
        let compiled_patterns = Self::compile_patterns(&patterns);
        let keyword_matcher = Self::build_keyword_matcher();
        
        Ok(Self {
            patterns: compiled_patterns,
            keyword_matcher,
        })
    }

    fn compile_patterns(patterns: &[MaskingPattern]) -> Vec<CompiledPattern> {
        let mut compiled = Vec::new();
        
        for pattern in patterns {
            match Regex::new(&pattern.regex) {
                Ok(regex) => {
                    compiled.push(CompiledPattern {
                        name: pattern.name.clone(),
                        regex,
                        replacement: pattern.replacement.clone(),
                        enabled: pattern.enabled,
                        priority: pattern.priority,
                        category: pattern.category.clone(),
                    });
                }
                Err(e) => {
                    warn!("Failed to compile pattern '{}': {}", pattern.name, e);
                }
            }
        }
        
        // Sort by priority (higher first)
        compiled.sort_by(|a, b| b.priority.cmp(&a.priority));
        compiled
    }

    fn build_keyword_matcher() -> Option<AhoCorasick> {
        AhoCorasick::new(&*SENSITIVE_KEYWORDS).ok()
    }

    pub async fn mask_text(&self, text: &str, context: Option<&str>) -> Result<MaskingResult, MaskingError> {
        if text.len() > 100_000 {
            return Err(MaskingError::TextTooLarge { 
                size: text.len(), 
                max: 100_000 
            });
        }

        let mut masked_text = text.to_string();
        let mut detected_patterns = Vec::new();
        let mut pattern_counts = HashMap::new();

        // Apply context-aware masking if context is provided
        if let Some(ctx) = context {
            if self.is_sensitive_context(ctx) {
                debug!("Applying enhanced masking for sensitive context: {}", ctx);
                masked_text = self.apply_enhanced_masking(&masked_text);
            }
        }

        // Apply PII patterns
        for pattern in &self.patterns {
            if !pattern.enabled {
                continue;
            }

            let matches: Vec<_> = pattern.regex.find_iter(&masked_text).collect();
            if !matches.is_empty() {
                detected_patterns.push(pattern.name.clone());
                pattern_counts.insert(pattern.name.clone(), matches.len());
                
                debug!("Found {} matches for pattern '{}'", matches.len(), pattern.name);
                
                // Replace all matches
                masked_text = pattern.regex.replace_all(&masked_text, &pattern.replacement).to_string();
            }
        }

        // Apply keyword-based sensitive data detection
        if let Some(matcher) = &self.keyword_matcher {
            if matcher.find_iter(&text.to_lowercase()).next().is_some() {
                detected_patterns.push("sensitive_keyword".to_string());
                debug!("Detected sensitive keywords in text");
            }
        }

        Ok(MaskingResult {
            masked_text,
            detected_patterns,
            pattern_counts,
        })
    }

    fn is_sensitive_context(&self, context: &str) -> bool {
        let context_lower = context.to_lowercase();
        context_lower.contains("password") 
            || context_lower.contains("login")
            || context_lower.contains("auth")
            || context_lower.contains("credential")
            || context_lower.contains("bank")
            || context_lower.contains("payment")
    }

    fn apply_enhanced_masking(&self, text: &str) -> String {
        // More aggressive masking for sensitive contexts
        let enhanced_patterns = vec![
            (r"\b\w{4,}\b", "***"), // Mask all words longer than 3 characters
            (r"\d+", "***"),         // Mask all numbers
        ];

        let mut result = text.to_string();
        for (pattern, replacement) in enhanced_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                result = regex.replace_all(&result, replacement).to_string();
            }
        }
        result
    }

    pub fn get_patterns(&self) -> Vec<MaskingPattern> {
        self.patterns.iter().map(|p| MaskingPattern {
            name: p.name.clone(),
            regex: "***".to_string(), // Don't expose actual regex for security
            replacement: p.replacement.clone(),
            enabled: p.enabled,
            priority: p.priority,
            category: p.category.clone(),
        }).collect()
    }

    pub fn update_patterns(&mut self, patterns: Vec<MaskingPattern>) -> Result<(), MaskingError> {
        let compiled = Self::compile_patterns(&patterns);
        self.patterns = compiled;
        Ok(())
    }
}

impl Default for MaskingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio_test::async_test]
    async fn test_cpf_masking() {
        let engine = MaskingEngine::new();
        let result = engine.mask_text("Meu CPF é 123.456.789-00", None).await.unwrap();
        
        assert!(result.masked_text.contains("***.***.***-**"));
        assert!(result.detected_patterns.contains(&"cpf".to_string()));
    }

    #[tokio_test::async_test]
    async fn test_email_masking() {
        let engine = MaskingEngine::new();
        let result = engine.mask_text("Contato: user@example.com", None).await.unwrap();
        
        assert!(result.masked_text.contains("***@***.***"));
        assert!(result.detected_patterns.contains(&"email".to_string()));
    }

    #[tokio_test::async_test]
    async fn test_multiple_patterns() {
        let engine = MaskingEngine::new();
        let text = "CPF: 123.456.789-00, Email: test@gmail.com, Telefone: (11) 99999-9999";
        let result = engine.mask_text(text, None).await.unwrap();
        
        assert!(result.detected_patterns.len() >= 2);
        assert!(result.pattern_counts.len() >= 2);
    }

    #[tokio_test::async_test]
    async fn test_sensitive_context() {
        let engine = MaskingEngine::new();
        let result = engine.mask_text("mypassword123", Some("Password Field")).await.unwrap();
        
        // Should apply enhanced masking for sensitive context
        assert!(!result.masked_text.contains("mypassword123"));
    }
} 