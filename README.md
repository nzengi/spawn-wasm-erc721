# spawn-wasm-erc721

`spawn-wasm-erc721` is a Rust library designed to handle ownership and user roles for smart contracts in a WebAssembly (WASM) environment. This library provides essential functionality for role management, including assigning roles to users, checking role-based access, and transferring ownership.

## Features

- **Ownership Management**: Enables ownership transfer and ensures that only the owner can perform sensitive actions like role assignments.
- **Role-based Access Control**: Allows access to contract methods based on assigned roles.
- **WASM Compatibility**: Designed to work seamlessly in WebAssembly, allowing for easy integration into JavaScript-based environments.
- **Event Logging**: Logs important events like ownership transfers and role assignments to the browser console for easier debugging.

## Installation

To use the library, first add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console"] }
```

Also, ensure the following settings are added to your Cargo.toml for WebAssembly compilation:

```toml
[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-Oz"]
```
## Usage
Initialize RoleManager
To initialize the RoleManager struct with an owner:

```rust
use spawn_wasm_erc721::RoleManager;

let owner = "owner1".to_string();
let mut role_manager = RoleManager::new(owner);
```

## Assign a Role
Assign a role to a user, ensuring the caller is the owner:

```rust
role_manager.assign_role("owner1".to_string(), "admin".to_string(), "user1".to_string());
```

## Check Role-Based Access
Check if a user has access based on a specific role:

```rust
let has_access = role_manager.role_based_access("user1".to_string(), "admin".to_string());
```

## Transfer Ownership
Ownership can only be transferred by the current owner:

```rust
role_manager.transfer_ownership("owner1".to_string(), "new_owner".to_string());
```

## List Role Users
To list all users with a specific role:

```rust
let users = role_manager.list_role_users("admin".to_string());
```





