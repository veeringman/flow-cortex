use crate::types::{AccountId, BankAccount, LedgerError, Token, TokenEvent, TokenMetadata, TokenStatus, TokenType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Simple in-memory ledger. Not thread-safe; the node will wrap it in a mutex if needed.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ledger {
    /// balances[account][token] = amount
    balances: HashMap<AccountId, HashMap<Token, u64>>,
    /// account that is allowed to mint tokens
    pub admin: AccountId,
    /// Token registry: symbol -> metadata
    pub tokens: HashMap<Token, TokenMetadata>,
    /// Token events for audit trail
    pub token_events: Vec<TokenEvent>,
    /// Bank accounts for settlement
    pub banks: HashMap<AccountId, BankAccount>,
    /// Last block height (for tracking when events occur)
    pub block_height: u64,
}

impl Ledger {
    pub fn new(admin: AccountId) -> Self {
        let mut ledger = Ledger {
            balances: HashMap::new(),
            admin: admin.clone(),
            tokens: HashMap::new(),
            token_events: Vec::new(),
            banks: HashMap::new(),
            block_height: 0,
        };
        
        // Initialize built-in tokens
        ledger.tokens.insert(
            "proof".to_string(),
            TokenMetadata {
                symbol: "proof".to_string(),
                name: "Proof Token".to_string(),
                decimals: 0,
                total_supply: 0,
                creator: admin.clone(),
                token_type: TokenType::Native,
                status: TokenStatus::Active,
                created_at: 0,
                metadata: None,
            },
        );
        
        ledger.tokens.insert(
            "flower".to_string(),
            TokenMetadata {
                symbol: "flower".to_string(),
                name: "Flow Dollar".to_string(),
                decimals: 6,
                total_supply: 0,
                creator: admin,
                token_type: TokenType::Stablecoin,
                status: TokenStatus::Active,
                created_at: 0,
                metadata: None,
            },
        );
        
        ledger
    }

    /// Return immutable reference to internal balances map.  Used by the node for
    /// snapshot root computation.
    pub fn all_balances(&self) -> &HashMap<AccountId, HashMap<Token, u64>> {
        &self.balances
    }

    /// create an account with zero balances
    pub fn create_account(&mut self, acct: &AccountId) {
        self.balances
            .entry(acct.clone())
            .or_insert_with(HashMap::new);
    }

    fn ensure_account(&self, acct: &AccountId) -> Result<(), LedgerError> {
        if self.balances.contains_key(acct) {
            Ok(())
        } else {
            Err(LedgerError::AccountNotFound(acct.clone()))
        }
    }

    pub fn balance(&self, acct: &AccountId, token: &Token) -> u64 {
        self.balances
            .get(acct)
            .and_then(|m| m.get(token))
            .cloned()
            .unwrap_or(0)
    }

    /// Mint tokens to an account. Only `self.admin` may perform this action.
    pub fn mint(
        &mut self,
        caller: &AccountId,
        to: &AccountId,
        token: Token,
        amount: u64,
    ) -> Result<(), LedgerError> {
        if caller != &self.admin {
            return Err(LedgerError::UnauthorizedMint);
        }
        self.create_account(to);
        let entry = self
            .balances
            .get_mut(to)
            .expect("account just created");
        *entry.entry(token).or_default() += amount;
        Ok(())
    }

    /// Transfer tokens between existing accounts.
    pub fn transfer(
        &mut self,
        from: &AccountId,
        to: &AccountId,
        token: Token,
        amount: u64,
    ) -> Result<(), LedgerError> {
        self.ensure_account(from)?;
        self.ensure_account(to)?;
        self.ensure_token_exists(&token)?;
        self.ensure_token_not_frozen(&token)?;
        
        let from_balance = self
            .balances
            .get_mut(from)
            .and_then(|m| m.get_mut(&token))
            .ok_or_else(|| LedgerError::InsufficientBalance { have: 0, need: amount })?;
        if *from_balance < amount {
            return Err(LedgerError::InsufficientBalance {
                have: *from_balance,
                need: amount,
            });
        }
        *from_balance -= amount;
        let to_entry = self.balances.get_mut(to).unwrap();
        *to_entry.entry(token).or_default() += amount;
        Ok(())
    }

    // ============== TOKEN MANAGEMENT ==============

    /// Create a new token in the registry (admin only)
    pub fn create_token(
        &mut self,
        caller: &AccountId,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: u64,
        token_type: TokenType,
        metadata: Option<String>,
    ) -> Result<(), LedgerError> {
        if caller != &self.admin {
            return Err(LedgerError::UnauthorizedMint);
        }
        
        if symbol.is_empty() {
            return Err(LedgerError::InvalidTokenSymbol(symbol));
        }
        
        let token = symbol.to_lowercase();
        if self.tokens.contains_key(&token) {
            return Err(LedgerError::TokenAlreadyExists(token));
        }
        
        // Create token metadata
        let token_meta = TokenMetadata {
            symbol: token.clone(),
            name: name.clone(),
            decimals,
            total_supply: initial_supply,
            creator: caller.clone(),
            token_type: token_type.clone(),
            status: TokenStatus::Active,
            created_at: self.block_height,
            metadata,
        };
        
        self.tokens.insert(token.clone(), token_meta);
        
        // Mint initial supply to admin
        if initial_supply > 0 {
            self.create_account(caller);
            let entry = self
                .balances
                .get_mut(caller)
                .expect("account just created");
            *entry.entry(token.clone()).or_default() += initial_supply;
        }
        
        // Log event
        self.token_events.push(TokenEvent::Created {
            symbol: token,
            creator: caller.clone(),
            name,
            decimals,
            block_height: self.block_height,
        });
        
        Ok(())
    }

    /// Burn tokens to reduce supply (remove from circulation)
    pub fn burn(
        &mut self,
        caller: &AccountId,
        token: &Token,
        from: &AccountId,
        amount: u64,
    ) -> Result<(), LedgerError> {
        // Only admin or the token creator can burn
        let token_meta = self.tokens.get(token).ok_or_else(|| LedgerError::TokenNotFound(token.clone()))?;
        if caller != &self.admin && caller != &token_meta.creator {
            return Err(LedgerError::UnauthorizedMint);
        }
        
        self.ensure_account(from)?;
        
        let from_balance = self
            .balances
            .get_mut(from)
            .and_then(|m| m.get_mut(token))
            .ok_or_else(|| LedgerError::InsufficientBalance { have: 0, need: amount })?;
        if *from_balance < amount {
            return Err(LedgerError::InsufficientBalance {
                have: *from_balance,
                need: amount,
            });
        }
        
        *from_balance -= amount;
        
        // Update total supply
        if let Some(meta) = self.tokens.get_mut(token) {
            meta.total_supply = meta.total_supply.saturating_sub(amount);
        }
        
        // Log event
        self.token_events.push(TokenEvent::Burned {
            symbol: token.clone(),
            from: from.clone(),
            amount,
            block_height: self.block_height,
        });
        
        Ok(())
    }

    /// Freeze a token (admin only) - prevents all transfers
    pub fn freeze_token(&mut self, caller: &AccountId, token: &Token) -> Result<(), LedgerError> {
        if caller != &self.admin {
            return Err(LedgerError::UnauthorizedMint);
        }
        
        let token_meta = self.tokens.get_mut(token).ok_or_else(|| LedgerError::TokenNotFound(token.clone()))?;
        token_meta.status = TokenStatus::Frozen;
        
        self.token_events.push(TokenEvent::Frozen {
            symbol: token.clone(),
            block_height: self.block_height,
        });
        
        Ok(())
    }

    /// Unfreeze a token (admin only) - re-enable transfers
    pub fn unfreeze_token(&mut self, caller: &AccountId, token: &Token) -> Result<(), LedgerError> {
        if caller != &self.admin {
            return Err(LedgerError::UnauthorizedMint);
        }
        
        let token_meta = self.tokens.get_mut(token).ok_or_else(|| LedgerError::TokenNotFound(token.clone()))?;
        token_meta.status = TokenStatus::Active;
        
        self.token_events.push(TokenEvent::Unfrozen {
            symbol: token.clone(),
            block_height: self.block_height,
        });
        
        Ok(())
    }

    /// Get token metadata
    pub fn get_token(&self, token: &Token) -> Option<&TokenMetadata> {
        self.tokens.get(token)
    }

    /// List all tokens
    pub fn list_tokens(&self) -> Vec<&TokenMetadata> {
        self.tokens.values().collect()
    }

    // ============== SETTLEMENT & BANKS ==============

    /// Approve a bank for settlement operations
    pub fn approve_bank(
        &mut self,
        caller: &AccountId,
        account_id: AccountId,
        bank_name: String,
        swift_code: String,
    ) -> Result<(), LedgerError> {
        if caller != &self.admin {
            return Err(LedgerError::UnauthorizedMint);
        }
        
        self.create_account(&account_id);
        
        let bank = BankAccount {
            account_id: account_id.clone(),
            bank_name,
            swift_code,
            is_approved: true,
            created_at: self.block_height,
            daily_mint_limits: HashMap::new(),
            daily_minted: HashMap::new(),
        };
        
        self.banks.insert(account_id, bank);
        Ok(())
    }

    /// Set daily mint limit for a bank on a specific token
    pub fn set_daily_limit(
        &mut self,
        caller: &AccountId,
        bank_account: &AccountId,
        token: &Token,
        daily_limit: u64,
    ) -> Result<(), LedgerError> {
        if caller != &self.admin {
            return Err(LedgerError::UnauthorizedMint);
        }
        
        let bank = self.banks.get_mut(bank_account)
            .ok_or_else(|| LedgerError::BankNotApproved(bank_account.clone()))?;
        
        bank.daily_mint_limits.insert(token.clone(), daily_limit);
        Ok(())
    }

    /// Settlement mint: Bank requests to mint stablecoins
    pub fn settlement_mint(
        &mut self,
        caller: &AccountId,
        token: &Token,
        amount: u64,
        reference: String,
    ) -> Result<(), LedgerError> {
        // Validate caller is a bank
        let bank = self.banks.get(caller)
            .ok_or_else(|| LedgerError::BankNotApproved(caller.clone()))?;
        
        if !bank.is_approved {
            return Err(LedgerError::BankNotApproved(caller.clone()));
        }
        
        // Check token exists
        self.ensure_token_exists(token)?;
        
        // Check daily limit
        if let Some(limit) = bank.daily_mint_limits.get(token) {
            let minted = bank.daily_minted.get(token).cloned().unwrap_or(0);
            if minted + amount > *limit {
                return Err(LedgerError::DailyLimitExceeded {
                    bank: caller.clone(),
                    limit: *limit,
                    minted: minted + amount,
                });
            }
        }
        
        // Update daily counter
        let bank = self.banks.get_mut(caller).unwrap();
        let entry = bank.daily_minted.entry(token.clone()).or_default();
        *entry += amount;
        
        // Mint tokens to bank account
        self.create_account(caller);
        let acc_entry = self.balances.get_mut(caller).unwrap();
        *acc_entry.entry(token.clone()).or_default() += amount;
        
        // Update total supply
        if let Some(meta) = self.tokens.get_mut(token) {
            meta.total_supply += amount;
        }
        
        // Log event
        self.token_events.push(TokenEvent::Minted {
            symbol: token.clone(),
            to: caller.clone(),
            amount,
            block_height: self.block_height,
        });
        
        Ok(())
    }

    /// Settlement burn: Bank requests to burn (redeem) stablecoins
    pub fn settlement_burn(
        &mut self,
        caller: &AccountId,
        token: &Token,
        amount: u64,
        reference: String,
    ) -> Result<(), LedgerError> {
        // Validate caller is a bank
        let bank = self.banks.get(caller)
            .ok_or_else(|| LedgerError::BankNotApproved(caller.clone()))?;
        
        if !bank.is_approved {
            return Err(LedgerError::BankNotApproved(caller.clone()));
        }
        
        // Check token exists
        self.ensure_token_exists(token)?;
        
        // Use burn operation
        self.burn(&self.admin.clone(), token, caller, amount)?;
        
        Ok(())
    }

    /// Settlement transfer: Bank-to-bank transfer
    pub fn settlement_transfer(
        &mut self,
        from_account: &AccountId,
        to_account: &AccountId,
        token: &Token,
        amount: u64,
        reference: String,
    ) -> Result<(), LedgerError> {
        // Validate both are approved banks
        let from_bank = self.banks.get(from_account)
            .ok_or_else(|| LedgerError::BankNotApproved(from_account.clone()))?;
        
        if !from_bank.is_approved {
            return Err(LedgerError::BankNotApproved(from_account.clone()));
        }
        
        let to_bank = self.banks.get(to_account)
            .ok_or_else(|| LedgerError::BankNotApproved(to_account.clone()))?;
        
        if !to_bank.is_approved {
            return Err(LedgerError::BankNotApproved(to_account.clone()));
        }
        
        // Check token exists and not frozen
        self.ensure_token_exists(token)?;
        self.ensure_token_not_frozen(token)?;
        
        // Perform transfer using existing transfer method
        self.transfer(from_account, to_account, token.clone(), amount)?;
        
        Ok(())
    }

    // ============== HELPERS ==============

    fn ensure_token_exists(&self, token: &Token) -> Result<(), LedgerError> {
        if self.tokens.contains_key(token) {
            Ok(())
        } else {
            Err(LedgerError::TokenNotFound(token.clone()))
        }
    }

    fn ensure_token_not_frozen(&self, token: &Token) -> Result<(), LedgerError> {
        if let Some(meta) = self.tokens.get(token) {
            if meta.status == TokenStatus::Frozen {
                return Err(LedgerError::TokenFrozen(token.clone()));
            }
        }
        Ok(())
    }
}

impl Ledger {
    /// write current ledger to json file at `path`
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let encoded = serde_json::to_string_pretty(self)?;
        fs::write(path, encoded)
    }

    /// load ledger from json file
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let ledger = serde_json::from_str(&data)?;
        Ok(ledger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Token;

    #[test]
    fn mint_and_transfer() {
        let admin = "admin".to_string();
        let mut ledger = Ledger::new(admin.clone());
        ledger.create_account(&"alice".to_string());
        ledger.create_account(&"bob".to_string());

        // admin can mint
        assert!(ledger
            .mint(&admin, &"alice".to_string(), "proof".to_string(), 100)
            .is_ok());
        assert_eq!(ledger.balance(&"alice".to_string(), &"proof".to_string()), 100);

        // non-admin cannot mint
        assert!(matches!(
            ledger.mint(&"alice".to_string(), &"bob".to_string(), "flower".to_string(), 50),
            Err(LedgerError::UnauthorizedMint)
        ));

        // transfer fails with insufficient funds
        assert!(matches!(
            ledger.transfer(&"alice".to_string(), &"bob".to_string(), "proof".to_string(), 200),
            Err(LedgerError::InsufficientBalance { .. })
        ));

        // transfer success
        assert!(ledger
            .transfer(&"alice".to_string(), &"bob".to_string(), "proof".to_string(), 30)
            .is_ok());
        assert_eq!(ledger.balance(&"alice".to_string(), &"proof".to_string()), 70);
        assert_eq!(ledger.balance(&"bob".to_string(), &"proof".to_string()), 30);
    }
}
