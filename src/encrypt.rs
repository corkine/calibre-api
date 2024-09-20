use ring::{rand as ring_rand, pbkdf2};
use data_encoding::HEXLOWER;

const SALT_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const DEFAULT_ITERATIONS: u32 = 600000;

fn generate_salt(length: usize) -> String {
    let rng = ring_rand::SystemRandom::new();
    (0..length)
        .map(|_| {
            let mut byte = [0u8; 1];
            ring_rand::SecureRandom::fill(&rng, &mut byte).expect("RNG failure");
            SALT_CHARS[byte[0] as usize % SALT_CHARS.len()] as char
        })
        .collect()
}

#[allow(dead_code)]
pub fn generate_password_hash(password: &str, method: &str, salt_length: usize) -> String {
    let salt = generate_salt(salt_length);
    let (hash, actual_method) = hash_internal(method, &salt, password);
    format!("{}${}${}", actual_method, salt, hash)
}

pub fn check_password_hash(pwhash: &str, password: &str) -> bool {
    let parts: Vec<&str> = pwhash.split('$').collect();
    if parts.len() != 3 {
        return false;
    }
    let method = parts[0];
    let salt = parts[1];
    let hash = parts[2];
    
    let (computed_hash, _) = hash_internal(method, salt, password);
    computed_hash == hash
}

fn hash_internal(method: &str, salt: &str, password: &str) -> (String, String) {
    if method.starts_with("pbkdf2:") {
        let parts: Vec<&str> = method.split(':').collect();
        if parts.len() < 2 || parts.len() > 3 {
            panic!("Invalid PBKDF2 method format");
        }
        let hash_method = parts[1];
        let iterations = if parts.len() == 3 {
            parts[2].parse().unwrap_or(DEFAULT_ITERATIONS)
        } else {
            DEFAULT_ITERATIONS
        };
        
        match hash_method {
            "sha256" => {
                let mut result = [0u8; 32];
                pbkdf2::derive(
                    pbkdf2::PBKDF2_HMAC_SHA256,
                    std::num::NonZeroU32::new(iterations).unwrap(),
                    salt.as_bytes(),
                    password.as_bytes(),
                    &mut result,
                );
                (HEXLOWER.encode(&result), format!("pbkdf2:{}:{}", hash_method, iterations))
            },
            _ => panic!("Unsupported hash method"),
        }
    } else {
        panic!("Only PBKDF2 is supported in this implementation");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn main() {
        let password = "your_password";
        let hashed = generate_password_hash(password, "pbkdf2:sha256", 8);
        println!("Hashed password: {}", hashed);

        let is_valid = check_password_hash(&hashed, password);
        assert!(is_valid);

        let is_invalid = check_password_hash(&hashed, "wrong_password");
        assert!(!is_invalid);
    }

    #[test]
    fn test_gen() {
        let valid = check_password_hash(
            "pbkdf2:sha256:600000$hQdi8AMq$813059eab6f37215457e25c289b5ba811564b1b65c1a86f747391d7fc4ab092a", 
    "!123456");
        assert!(valid);
    }
}
