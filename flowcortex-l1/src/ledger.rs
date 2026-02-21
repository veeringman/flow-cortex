use crate::types::{AccountId, LedgerError, Token};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Simple in-memory ledger. Not thread-safe; the node will wrap it in a mutex if needed.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ledger {
    /// balances[account][token] = amount
    balances: HashMap<AccountId, HashMap<Token, u64>>,
    /// account that is allowed to mint new tokens
    pub admin: AccountId,
}

impl Ledger {
    pub fn new(admin: AccountId) -> Self {
        Ledger {
            balances: HashMap::new(),
            admin,
        }
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
            .mint(&admin, &"alice".to_string(), Token::Proof, 100)
            .is_ok());
        assert_eq!(ledger.balance(&"alice".to_string(), &Token::Proof), 100);

        // non-admin cannot mint
        assert!(matches!(
            ledger.mint(&"alice".to_string(), &"bob".to_string(), Token::FloweR, 50),
            Err(LedgerError::UnauthorizedMint)
        ));

        // transfer fails with insufficient funds
        assert!(matches!(
            ledger.transfer(&"alice".to_string(), &"bob".to_string(), Token::Proof, 200),
            Err(LedgerError::InsufficientBalance { .. })
        ));

        // transfer success
        assert!(ledger
            .transfer(&"alice".to_string(), &"bob".to_string(), Token::Proof, 30)
            .is_ok());
        assert_eq!(ledger.balance(&"alice".to_string(), &Token::Proof), 70);
        assert_eq!(ledger.balance(&"bob".to_string(), &Token::Proof), 30);
    }
}
