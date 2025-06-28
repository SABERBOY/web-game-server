use sha2::{Sha256, Digest};
use hex;

pub struct PasswordUtil;

impl PasswordUtil {
    /// 对密码进行哈希处理
    pub fn hash_password(password: &str) -> Result<String, std::convert::Infallible> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// 验证密码是否匹配
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, std::convert::Infallible> {
        let password_hash = Self::hash_password(password)?;
        Ok(password_hash == hash)
    }
}