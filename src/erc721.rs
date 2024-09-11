use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::console;

/// ERC721 Token standardına uygun NFT yönetimi
#[wasm_bindgen]
pub struct ERC721 {
    owner: String,
    token_owner: HashMap<u64, String>, // Token ID -> Sahip Adresi
    owned_tokens: HashMap<String, Vec<u64>>, // Kullanıcı Adresi -> Sahip Olduğu Tokenlar
    approvals: HashMap<u64, String>, // Token ID -> Onaylı Adres
}

#[wasm_bindgen]
impl ERC721 {
    /// Yeni bir ERC721 kontratı oluşturur
    #[wasm_bindgen(constructor)]
    pub fn new(owner: String) -> ERC721 {
        Self::log_event("ERC721 Created", &format!("Owner: {}", owner));
        ERC721 {
            owner,
            token_owner: HashMap::new(),
            owned_tokens: HashMap::new(),
            approvals: HashMap::new(),
        }
    }

    /// Token'ın sahibini döner
    pub fn owner_of(&self, token_id: u64) -> Option<String> {
        self.token_owner.get(&token_id).cloned()
    }

    /// Bir token'ı mint'ler ve sahibine atar (sadece kontrat sahibi yapabilir)
    pub fn mint(&mut self, owner: String, token_id: u64) -> Result<(), String> {
        if owner != self.owner {
            Self::log_event("Minting Failed", "Unauthorized attempt");
            return Err("Only the contract owner can mint new tokens".to_string());
        }

        if self.token_owner.contains_key(&token_id) {
            Self::log_event("Minting Failed", &format!("Token ID {} already exists", token_id));
            return Err("Token ID already exists".to_string());
        }

        self.token_owner.insert(token_id, owner.clone());
        self.owned_tokens.entry(owner.clone()).or_insert(Vec::new()).push(token_id);
        Self::log_event("Token Minted", &format!("Token ID: {}, Owner: {}", token_id, owner));
        Ok(())
    }

    /// Token'ı başka bir kullanıcıya transfer eder
    pub fn transfer(&mut self, from: String, to: String, token_id: u64) -> Result<(), String> {
        let owner = self.token_owner.get(&token_id).ok_or("Token does not exist")?;

        if owner != &from && !self.is_approved_or_owner(from.clone(), token_id) {
            Self::log_event("Transfer Failed", "Unauthorized attempt");
            return Err("Unauthorized transfer attempt".to_string());
        }

        self.remove_token_from_owner(from.clone(), token_id);
        self.token_owner.insert(token_id, to.clone());
        self.owned_tokens.entry(to.clone()).or_insert(Vec::new()).push(token_id);
        Self::log_event("Token Transferred", &format!("Token ID: {}, From: {}, To: {}", token_id, from, to));
        Ok(())
    }

    /// Token'ı başka bir kullanıcıya transfer edebilmesi için onay verir
    pub fn approve(&mut self, owner: String, approved: String, token_id: u64) -> Result<(), String> {
        let token_owner = self.token_owner.get(&token_id).ok_or("Token does not exist")?;

        if token_owner != &owner {
            Self::log_event("Approval Failed", "Unauthorized attempt");
            return Err("Only the owner can approve".to_string());
        }

        self.approvals.insert(token_id, approved.clone());
        Self::log_event("Approval Granted", &format!("Token ID: {}, Approved for: {}", token_id, approved));
        Ok(())
    }

    /// Bir token'ın kime onaylı olduğunu döner
    pub fn get_approved(&self, token_id: u64) -> Option<String> {
        self.approvals.get(&token_id).cloned()
    }

    /// Token sahibinin onaylayıp onaylamadığını kontrol eder
    pub fn is_approved_or_owner(&self, user: String, token_id: u64) -> bool {
        let owner = self.token_owner.get(&token_id);
        let approved = self.approvals.get(&token_id);

        owner.map(|o| o == &user).unwrap_or(false) || approved.map(|a| a == &user).unwrap_or(false)
    }

    /// Kullanıcıya ait olan tüm token'ları listeler
    pub fn tokens_of_owner(&self, owner: String) -> Vec<u64> {
        self.owned_tokens.get(&owner).cloned().unwrap_or_else(Vec::new)
    }

    /// Olayları tarayıcı konsoluna loglar
    fn log_event(event: &str, details: &str) {
        console::log_2(&event.into(), &details.into());
    }

    /// Token sahibinden token'ı kaldırır (Transfer sırasında kullanılır)
    fn remove_token_from_owner(&mut self, owner: String, token_id: u64) {
        if let Some(tokens) = self.owned_tokens.get_mut(&owner) {
            tokens.retain(|&id| id != token_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_and_transfer() {
        let mut erc721 = ERC721::new("owner".to_string());

        // Mint token
        assert!(erc721.mint("owner".to_string(), 1).is_ok());
        assert_eq!(erc721.owner_of(1).unwrap(), "owner".to_string());

        // Transfer token
        assert!(erc721.transfer("owner".to_string(), "user1".to_string(), 1).is_ok());
        assert_eq!(erc721.owner_of(1).unwrap(), "user1".to_string());

        // Unauthorized transfer should fail
        assert!(erc721.transfer("owner".to_string(), "user2".to_string(), 1).is_err());
    }

    #[test]
    fn test_approval_and_transfer() {
        let mut erc721 = ERC721::new("owner".to_string());

        // Mint token
        erc721.mint("owner".to_string(), 1).unwrap();

        // Approve transfer
        assert!(erc721.approve("owner".to_string(), "user1".to_string(), 1).is_ok());
        assert_eq!(erc721.get_approved(1).unwrap(), "user1".to_string());

        // Approved user can transfer
        assert!(erc721.transfer("user1".to_string(), "user2".to_string(), 1).is_ok());
        assert_eq!(erc721.owner_of(1).unwrap(), "user2".to_string());
    }
}
