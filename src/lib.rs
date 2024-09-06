use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// ERC-721 token yapısı
#[wasm_bindgen]
pub struct ERC721 {
    owner: String,  // Kontratın sahibi
    name: String,
    symbol: String,
    owner_of: HashMap<u64, String>,   // Token ID -> Sahip Adresi
    balances: HashMap<String, u64>,   // Kullanıcı -> Token Sayısı
    token_approvals: HashMap<u64, String>,  // Token ID -> Onaylanan Adres
    operator_approvals: HashMap<String, HashMap<String, bool>>,  // Kullanıcı -> Operatör -> Onay Durumu
    token_uri: HashMap<u64, String>,  // Token ID -> Metadata URI
    burned_tokens: HashMap<u64, bool>,  // Yakılan tokenlar
}

#[wasm_bindgen]
impl ERC721 {
    /// Yeni bir ERC-721 kontratı oluşturur. Bu fonksiyon kontratı dağıtan kişiyi kontrat sahibi olarak belirler.
    #[wasm_bindgen(constructor)]
    pub fn new(owner: &str, name: &str, symbol: &str) -> ERC721 {
        ERC721 {
            owner: owner.to_string(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            owner_of: HashMap::new(),
            balances: HashMap::new(),
            token_approvals: HashMap::new(),
            operator_approvals: HashMap::new(),
            token_uri: HashMap::new(),
            burned_tokens: HashMap::new(),
        }
    }

    /// Kontrat sahibini kontrol eder.
    fn only_owner(&self, caller: &str) -> bool {
        self.owner == caller
    }

    /// Olayları loglamak için basit bir log fonksiyonu
    fn log_event(event_name: &str, details: &str) {
        web_sys::console::log_2(&event_name.into(), &details.into());
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

    /// Token mint işlemi (oluşturma). Bu işlem sadece kontrat sahibi tarafından yapılabilir.
    pub fn mint(&mut self, caller: &str, recipient: &str, token_id: u64, token_uri: &str) -> bool {
        if token_id == 0 {
            return false; // Geçersiz token ID
        }

        // Kontrat sahibi kontrolü
        if !self.only_owner(caller) {
            return false;  // Sadece kontrat sahibi mint yapabilir
        }

        // Token ID'nin daha önce mint edilmediğini kontrol et
        if self.owner_of.contains_key(&token_id) {
            return false;  // Token ID zaten mevcut
        }

        // Token sahibini ve bakiyesini güncelle
        self.owner_of.insert(token_id, recipient.to_string());
        let recipient_balance = self.balances.get(recipient).unwrap_or(&0);
        self.balances.insert(recipient.to_string(), recipient_balance + 1);

        // Token URI'sini ekle
        self.token_uri.insert(token_id, token_uri.to_string());

        // Olayı logla
        Self::log_event("Mint", &format!("TokenID: {}, Recipient: {}", token_id, recipient));

        true
    }

    /// Token transferi yapar. Sadece token sahibi, onaylanmış adresler veya tüm tokenlar için yetkilendirilmiş operatörler transfer yapabilir.
    pub fn transfer(&mut self, from: &str, to: &str, token_id: u64) -> bool {
        // Tokenin var olup olmadığını ve sahibini kontrol et
        if let Some(owner) = self.owner_of.get(&token_id) {
            // Token sahibi ya da onaylanan adres veya operatör kontrolü
            if owner != from && self.get_approved(token_id) != Some(from.to_string()) 
                && !self.is_approved_for_all(owner, from) {
                return false;  // Token sahibi, onaylı adres veya operatör değil
            }
        } else {
            return false;  // Token mevcut değil
        }

        // Bakiye kontrolü
        let from_balance = *self.balances.get(from).unwrap_or(&0);
        if from_balance == 0 {
            return false;  // Bakiye yetersiz
        }

        // Sahipliği değiştir ve bakiyeleri güncelle
        self.owner_of.insert(token_id, to.to_string());
        let to_balance = *self.balances.get(to).unwrap_or(&0);

        self.balances.insert(from.to_string(), from_balance - 1);
        self.balances.insert(to.to_string(), to_balance + 1);

        // Önceki onaylı adresi sıfırla
        self.token_approvals.remove(&token_id);

        // Olayı logla
        Self::log_event("Transfer", &format!("From: {}, To: {}, TokenID: {}", from, to, token_id));

        true
    }

    /// Token ID için onaylı adresi döndürür.
    pub fn get_approved(&self, token_id: u64) -> Option<String> {
        self.token_approvals.get(&token_id).cloned()
    }

    /// Token için onay verir. `token_id` tokeninin `approved_address` adresine transferine izin verir. 
    /// Bu işlem sadece token sahibi tarafından yapılabilir.
    pub fn approve(&mut self, caller: &str, token_id: u64, approved_address: &str) -> bool {
        if let Some(owner) = self.owner_of.get(&token_id) {
            if owner != caller {
                return false;  // Sadece token sahibi onay verebilir
            }
            self.token_approvals.insert(token_id, approved_address.to_string());

            // Olayı logla
            Self::log_event("Approval", &format!("TokenID: {}, ApprovedAddress: {}", token_id, approved_address));
            return true;
        }
        false
    }

    /// Tüm tokenler için bir operatör belirler. Bu operatör, belirlenen kullanıcının tüm tokenları için işlem yapabilir.
    pub fn set_approval_for_all(&mut self, owner: &str, operator: &str, approved: bool) {
        let approvals = self.operator_approvals.entry(owner.to_string()).or_insert(HashMap::new());
        approvals.insert(operator.to_string(), approved);

        // Olayı logla
        Self::log_event("ApprovalForAll", &format!("Owner: {}, Operator: {}, Approved: {}", owner, operator, approved));
    }

    /// Bir operatörün tüm tokenlar için onaylı olup olmadığını kontrol eder.
    pub fn is_approved_for_all(&self, owner: &str, operator: &str) -> bool {
        self.operator_approvals.get(owner)
            .and_then(|approvals| approvals.get(operator))
            .cloned()
            .unwrap_or(false)
    }

    /// Bir operatörün onayını kaldırır.
    pub fn revoke_operator(&mut self, owner: &str, operator: &str) -> bool {
        if let Some(approvals) = self.operator_approvals.get_mut(owner) {
            if approvals.contains_key(operator) {
                approvals.remove(operator);
                
                // Olayı logla
                Self::log_event("RevokeOperator", &format!("Owner: {}, Operator: {}", owner, operator));
                return true;
            }
        }
        false
    }

    /// Token yakma işlemi (burn). Token yok edilir ve sahibi artık o tokena sahip olmaz. Yakılan tokenlar `burned_tokens` içinde saklanır.
    pub fn burn(&mut self, caller: &str, token_id: u64) -> bool {
        if let Some(owner) = self.owner_of.get(&token_id) {
            // Token sahibi veya onaylı olup olmadığını kontrol et
            if owner != caller && !self.is_approved_for_all(owner, caller) {
                return false;  // Token sahibi veya onaylı değil
            }

            // Tokenı kaldır ve bakiye güncelle
            self.owner_of.remove(&token_id);
            let owner_balance = *self.balances.get(&owner).unwrap_or(&0);
            self.balances.insert(owner.clone(), owner_balance.saturating_sub(1));

            // Token metadata URI'sini sil
            self.token_uri.remove(&token_id);

            // Yakılan tokenı sakla
            self.burned_tokens.insert(token_id, true);

            // Olayı logla
            Self::log_event("Burn", &format!("TokenID: {}, Owner: {}", token_id, owner));

            return true;
        }
        false
    }

    /// Tokenin metadata URI'sini döndürür.
    pub fn token_uri(&self, token_id: u64) -> Option<String> {
        self.token_uri.get(&token_id).cloned()
    }

    /// Kontrat sahibini değiştirme. Bu işlem sadece mevcut kontrat sahibi tarafından yapılabilir.
    pub fn transfer_ownership(&mut self, caller: &str, new_owner: &str) -> bool {
        if !self.only_owner(caller) {
            return false;  // Sadece mevcut sahip değiştirme yapabilir
        }
        self.owner = new_owner.to_string();

        // Olayı logla
        Self::log_event("OwnershipTransferred", &format!("NewOwner: {}", new_owner));

        true
    }
}
