use lazy_static::lazy_static;
use regex::Regex;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use validator::{ValidationError, ValidationErrorsKind};

lazy_static! {
    pub static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9_-]{4,50}$").unwrap();
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut has_whitespace = false;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;

    for c in password.chars() {
        has_whitespace |= c.is_whitespace();
        has_lower |= c.is_lowercase();
        has_upper |= c.is_uppercase();
        has_digit |= c.is_digit(10);
    }

    if !has_whitespace && has_upper && has_lower && has_digit && password.len() >= 8 {
        Ok(())
    } else {
        return Err(ValidationError::new("Password Validation Failed"));
    }
}

pub async fn username_exists(username: &str, pool: &Pool<Postgres>) -> bool {
    let res = sqlx::query!("SELECT username FROM users WHERE username = $1", username)
        .fetch_one(pool)
        .await;

    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn email_exists(email: &str, pool: &Pool<Postgres>) -> bool {
    let res = sqlx::query!("SELECT email FROM users WHERE email = $1", email)
        .fetch_one(pool)
        .await;

    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn extract_errors(
    errors: &HashMap<&'static str, ValidationErrorsKind>,
) -> HashMap<String, String> {
    let mut extracted_errs: HashMap<String, String> = HashMap::new();
    for (k, v) in errors {
        match v {
            ValidationErrorsKind::Struct(_) => todo!(),
            ValidationErrorsKind::List(_) => {}
            ValidationErrorsKind::Field(errs) => {
                for err in errs {
                    let msg = err.message.as_ref().unwrap();
                    extracted_errs.insert(k.to_string(), msg.to_string());
                }
            }
        }
    }
    extracted_errs
}
