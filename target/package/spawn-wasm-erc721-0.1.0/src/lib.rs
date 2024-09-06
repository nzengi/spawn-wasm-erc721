use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// ERC-721 Non-Fungible Token Standardı
#[wasm_bindgen]
pub struct ERC721 {
    name: String,
    symbol: String,
    owner_of: HashMap<u64, String>,   // Token ID -> Sahip Adresi
    balances: HashMap<String, u64>,   // Kullanıcı -> Token Sayısı
    token_approvals: HashMap<u64, String>,  // Token ID -> Onaylanan Adres
    operator_approvals: HashMap<String, HashMap<String, bool>>,  // Sahip -> Operatör -> Onay Durumu
    token_uri: HashMap<u64, String>,  // Token ID -> Metadata URI
}

#[wasm_bindgen]
impl ERC721 {
    /// Yeni bir ERC-721 token oluşturur.
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, symbol: &str) -> ERC721 {
        ERC721 {
            name: name.to_string(),
            symbol: symbol.to_string(),
            owner_of: HashMap::new(),
            balances: HashMap::new(),
            token_approvals: HashMap::new(),
            operator_approvals: HashMap::new(),
            token_uri: HashMap::new(),
        }
    }

    /// Token adını döndürür.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Token sembolünü döndürür.
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    /// Belirli bir tokenin sahibini sorgular.
    pub fn owner_of(&self, token_id: u64) -> Option<String> {
        self.owner_of.get(&token_id).cloned()
    }

    /// Bir kullanıcının kaç tane token sahibi olduğunu döndürür.
    pub fn balance_of(&self, owner: &str) -> u64 {
        *self.balances.get(owner).unwrap_or(&0)
    }

    /// Token transferi yapar. `from` adresinden `to` adresine `token_id` numaralı tokenı transfer eder.
    pub fn transfer(&mut self, from: &str, to: &str, token_id: u64) -> bool {
        // Token sahibini doğrula
        if let Some(owner) = self.owner_of.get(&token_id) {
            if owner != from {
                return false;  // Token sahibi değil
            }
        } else {
            return false;  // Token mevcut değil
        }

        // Sahipliği değiştir
        self.owner_of.insert(token_id, to.to_string());

        // Bakiye güncellemeleri için önceden değişkenleri al
        let from_balance = *self.balances.get(from).unwrap_or(&0);
        let to_balance = *self.balances.get(to).unwrap_or(&0);

        // Bakiye güncellemesi
        self.balances.insert(from.to_string(), from_balance - 1);
        self.balances.insert(to.to_string(), to_balance + 1);

        true
    }

    /// Token için onay verir. `token_id` tokeninin `approved_address` adresine transferine izin verir.
    pub fn approve(&mut self, owner: &str, approved_address: &str, token_id: u64) -> bool {
        if let Some(token_owner) = self.owner_of.get(&token_id) {
            if token_owner == owner {
                self.token_approvals.insert(token_id, approved_address.to_string());
                return true;
            }
        }
        false
    }

    /// Onaylanan adresi sorgular.
    pub fn get_approved(&self, token_id: u64) -> Option<String> {
        self.token_approvals.get(&token_id).cloned()
    }

    /// Operatör için onay verir. Bir operatör, sahibin tüm tokenları için işlemler yapabilir.
    pub fn set_approval_for_all(&mut self, owner: &str, operator: &str, approved: bool) {
        let owner_approvals = self.operator_approvals.entry(owner.to_string()).or_insert(HashMap::new());
        owner_approvals.insert(operator.to_string(), approved);
    }

    /// Operatörün onay durumunu sorgular.
    pub fn is_approved_for_all(&self, owner: &str, operator: &str) -> bool {
        self.operator_approvals
            .get(owner)
            .and_then(|approvals| approvals.get(operator))
            .cloned()
            .unwrap_or(false)
    }

    /// Token mint işlemi (oluşturma). Tokenı `recipient` adresine atar ve metadata URI'sini ekler.
    pub fn mint(&mut self, recipient: &str, token_id: u64, token_uri: &str) -> bool {
        if self.owner_of.get(&token_id).is_none() {
            self.owner_of.insert(token_id, recipient.to_string());
            let recipient_balance = self.balances.get(recipient).unwrap_or(&0);
            self.balances.insert(recipient.to_string(), recipient_balance + 1);
            self.token_uri.insert(token_id, token_uri.to_string());
            return true;
        }
        false
    }

    /// Token burn işlemi (yakma). Tokenı kalıcı olarak yok eder.
    pub fn burn(&mut self, token_id: u64) -> bool {
        if let Some(owner) = self.owner_of.remove(&token_id) {
            let owner_balance = self.balances.get(&owner).unwrap_or(&0);
            self.balances.insert(owner, owner_balance - 1);
            self.token_uri.remove(&token_id);  // Metadata URI'sini de sil
            return true;
        }
        false
    }

    /// Tokenin metadata URI'sini döndürür.
    pub fn token_uri(&self, token_id: u64) -> Option<String> {
        self.token_uri.get(&token_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_and_owner_of() {
        let mut nft = ERC721::new("SpawnNFT", "SPWN");
        let success = nft.mint("user1", 1, "https://example.com/metadata/1");
        assert!(success);
        assert_eq!(nft.owner_of(1), Some("user1".to_string()));
        assert_eq!(nft.token_uri(1), Some("https://example.com/metadata/1".to_string()));
    }

    #[test]
    fn test_transfer() {
        let mut nft = ERC721::new("SpawnNFT", "SPWN");
        nft.mint("user1", 1, "https://example.com/metadata/1");
        let success = nft.transfer("user1", "user2", 1);
        assert!(success);
        assert_eq!(nft.owner_of(1), Some("user2".to_string()));
    }

    #[test]
    fn test_approve_and_get_approved() {
        let mut nft = ERC721::new("SpawnNFT", "SPWN");
        nft.mint("user1", 1, "https://example.com/metadata/1");
        let success = nft.approve("user1", "user2", 1);
        assert!(success);
        assert_eq!(nft.get_approved(1), Some("user2".to_string()));
    }

    #[test]
    fn test_set_and_is_approved_for_all() {
        let mut nft = ERC721::new("SpawnNFT", "SPWN");
        nft.mint("user1", 1, "https://example.com/metadata/1");
        nft.set_approval_for_all("user1", "operator1", true);
        assert!(nft.is_approved_for_all("user1", "operator1"));
    }

    #[test]
    fn test_burn() {
        let mut nft = ERC721::new("SpawnNFT", "SPWN");
        nft.mint("user1", 1, "https://example.com/metadata/1");
        let success = nft.burn(1);
        assert!(success);
        assert_eq!(nft.owner_of(1), None);
        assert_eq!(nft.token_uri(1), None);
    }
}
