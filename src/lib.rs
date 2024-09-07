use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use web_sys::console;

/// A library for managing ownership and roles in a contract
#[wasm_bindgen]
pub struct RoleManager {
    owner: String,
    roles: HashMap<String, HashSet<String>>, // Role -> Set of Users
}

#[wasm_bindgen]
impl RoleManager {
    /// Initializes a new contract with the owner and role management
    #[wasm_bindgen(constructor)]
    pub fn new(owner: String) -> RoleManager {
        Self::log_event("New RoleManager", &format!("Owner: {}", owner));
        RoleManager {
            owner,
            roles: HashMap::new(),
        }
    }

    /// Returns the current owner of the contract
    pub fn get_owner(&self) -> String {
        self.owner.clone()
    }

    /// Transfers ownership to a new user (only the current owner can call this)
    pub fn transfer_ownership(&mut self, current_owner: String, new_owner: String) -> Result<(), String> {
        if current_owner != self.owner {
            Self::log_event("Ownership Transfer Failed", "Unauthorized attempt");
            return Err("Only the current owner can transfer ownership".to_string());
        }
        self.owner = new_owner.clone();
        Self::log_event("Ownership Transferred", &format!("New Owner: {}", new_owner));
        Ok(())
    }

    /// Assigns a role to a specific user (only the owner can assign roles)
    pub fn assign_role(&mut self, owner: String, role: String, user: String) -> Result<(), String> {
        if owner != self.owner {
            Self::log_event("Assign Role Failed", "Unauthorized attempt");
            return Err("Only the owner can assign roles".to_string());
        }

        let role_users = self.roles.entry(role.clone()).or_insert(HashSet::new());
        if role_users.insert(user.clone()) {
            Self::log_event("Role Assigned", &format!("Role: {}, User: {}", role, user));
        } else {
            Self::log_event("Role Assignment Skipped", &format!("User: {} already has the role: {}", user, role));
        }
        Ok(())
    }

    /// Removes a role from a specific user (only the owner can remove roles)
    pub fn remove_role(&mut self, owner: String, role: String, user: String) -> Result<(), String> {
        if owner != self.owner {
            Self::log_event("Remove Role Failed", "Unauthorized attempt");
            return Err("Only the owner can remove roles".to_string());
        }

        if let Some(role_users) = self.roles.get_mut(&role) {
            if role_users.remove(&user) {
                Self::log_event("Role Removed", &format!("Role: {}, User: {}", role, user));
            } else {
                Self::log_event("Remove Role Skipped", &format!("User: {} does not have the role: {}", user, role));
            }
        }
        Ok(())
    }

    /// Checks if a user has a specific role
    pub fn has_role(&self, role: String, user: String) -> bool {
        if let Some(role_users) = self.roles.get(&role) {
            return role_users.contains(&user);
        }
        false
    }

    /// Checks if a user is the current owner
    pub fn is_owner(&self, user: String) -> bool {
        self.owner == user
    }

    /// Provides role-based access control (only the owner or users with the specified role can call this)
    pub fn role_based_access(&self, user: String, role: String) -> bool {
        self.is_owner(user.clone()) || self.has_role(role, user)
    }

    /// Lists all users assigned to a specific role
    pub fn list_role_users(&self, role: String) -> Vec<String> {
        self.roles
            .get(&role)
            .map(|users| users.iter().cloned().collect())
            .unwrap_or_else(Vec::new)
    }

    /// Logs events to the console for monitoring
    fn log_event(event: &str, details: &str) {
        console::log_2(&event.into(), &details.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_transfer() {
        let mut role_manager = RoleManager::new("owner".to_string());

        // Correct ownership transfer
        assert!(role_manager.transfer_ownership("owner".to_string(), "new_owner".to_string()).is_ok());
        assert_eq!(role_manager.get_owner(), "new_owner".to_string());

        // Unauthorized user tries to transfer ownership
        assert!(role_manager.transfer_ownership("wrong_user".to_string(), "owner".to_string()).is_err());
    }

    #[test]
    fn test_assign_role() {
        let mut role_manager = RoleManager::new("owner".to_string());

        // Successful role assignment
        assert!(role_manager.assign_role("owner".to_string(), "admin".to_string(), "user1".to_string()).is_ok());
        assert!(role_manager.has_role("admin".to_string(), "user1".to_string()));

        // Unauthorized user tries to assign a role
        assert!(role_manager.assign_role("wrong_user".to_string(), "admin".to_string(), "user2".to_string()).is_err());
    }

    #[test]
    fn test_remove_role() {
        let mut role_manager = RoleManager::new("owner".to_string());

        // Assign a role
        role_manager.assign_role("owner".to_string(), "admin".to_string(), "user1".to_string()).unwrap();
        assert!(role_manager.has_role("admin".to_string(), "user1".to_string()));

        // Successful role removal
        assert!(role_manager.remove_role("owner".to_string(), "admin".to_string(), "user1".to_string()).is_ok());
        assert!(!role_manager.has_role("admin".to_string(), "user1".to_string()));

        // Unauthorized user tries to remove a role
        assert!(role_manager.remove_role("wrong_user".to_string(), "admin".to_string(), "user1".to_string()).is_err());
    }
}
