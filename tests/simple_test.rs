#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_functionality() {
        // Teste básico de mascaramento de string
        let input = "123.456.789-01";
        let masked = mask_cpf(input);
        assert_eq!(masked, "***.***.***-01");
    }

    #[test]
    fn test_email_masking() {
        let input = "joao@example.com";
        let masked = mask_email(input);
        assert_eq!(masked, "j***@example.com");
    }

    #[test]
    fn test_phone_masking() {
        let input = "(11) 99999-1234";
        let masked = mask_phone(input);
        assert_eq!(masked, "(***) ***-****");
    }
}

// Funções auxiliares simples para testes
fn mask_cpf(cpf: &str) -> String {
    if cpf.len() >= 14 {
        format!("***.***.***-{}", &cpf[cpf.len()-2..])
    } else {
        "***.***.***-**".to_string()
    }
}

fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (username, domain) = email.split_at(at_pos);
        if username.len() > 1 {
            format!("{}***{}", &username[0..1], domain)
        } else {
            format!("***{}", domain)
        }
    } else {
        email.to_string()
    }
}

fn mask_phone(phone: &str) -> String {
    "(***) ***-****".to_string()
} 