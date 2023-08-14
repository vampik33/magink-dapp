#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod magink {
    use crate::ensure;
    use ink::storage::Mapping;
    use openbrush::{
        contracts::psp34::{
            extensions::mintable::psp34mintable_external::PSP34Mintable,
            Id,
            PSP34Error,
        },
        traits::String,
    };
    use wizard::WizardRef;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TooEarlyToClaim,
        UserNotFound,
    }

    #[ink(storage)]
    pub struct Magink {
        user: Mapping<AccountId, Profile>,
        wizard_contract: Option<WizardRef>,
    }

    #[derive(
        Debug, PartialEq, Eq, PartialOrd, Ord, Clone, scale::Encode, scale::Decode,
    )]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Profile {
        // duration in blocks until next claim
        claim_era: u8,
        // block number of last claim
        start_block: u32,
        // number of badges claimed
        badges_claimed: u8,
    }

    impl Magink {
        fn get_wizard_contract_code_hash() -> Hash {
            let mut bytes_array = [0u8; 32];
            if let Err(e) = hex::decode_to_slice(
                &"0x6dbe6fc60833f0f8cf4d68e8b178b6efb2498faa499d9b6b7389cfea5e80b559" // Don't like this hardcoded hash
                    [2..],
                &mut bytes_array,
            ) {
                panic!("Failed to convert: {:?}", e);
            }

            Hash::from(bytes_array)
        }

        /// Creates a new Magink smart contract.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                user: Mapping::new(),
                wizard_contract: None,
            }
        }

        #[ink(constructor)]
        pub fn new_with_hash(wizard_contract_code_hash: Hash) -> Self {
            let wizard_contract = WizardRef::new()
                .code_hash(wizard_contract_code_hash)
                .endowment(0)
                .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                .instantiate();

            Self {
                user: Mapping::new(),
                wizard_contract: Some(wizard_contract),
            }
        }

        #[ink(constructor)]
        pub fn new() -> Self {
            Self::new_with_hash(Self::get_wizard_contract_code_hash())
        }

        /// (Re)Start the Magink the claiming era for the caller.
        #[ink(message)]
        pub fn start(&mut self, era: u8) {
            let profile = Profile {
                claim_era: era,
                start_block: self.env().block_number(),
                badges_claimed: 0,
            };
            self.user.insert(self.env().caller(), &profile);
        }

        /// Claim the badge after the era.
        #[ink(message)]
        pub fn claim(&mut self) -> Result<(), Error> {
            ensure!(self.get_remaining() == 0, Error::TooEarlyToClaim);

            // update profile
            let mut profile = self.get_profile().ok_or(Error::UserNotFound).unwrap();
            profile.badges_claimed += 1;
            profile.start_block = self.env().block_number();
            self.user.insert(self.env().caller(), &profile);
            Ok(())
        }

        /// Returns the remaining blocks in the era.
        #[ink(message)]
        pub fn get_remaining(&self) -> u8 {
            let current_block = self.env().block_number();
            let caller = self.env().caller();
            self.user.get(&caller).map_or(0, |profile| {
                if current_block - profile.start_block >= profile.claim_era as u32 {
                    return 0
                }
                profile.claim_era - (current_block - profile.start_block) as u8
            })
        }

        /// Returns the remaining blocks in the era for the given account.
        #[ink(message)]
        pub fn get_remaining_for(&self, account: AccountId) -> u8 {
            let current_block = self.env().block_number();
            self.user.get(&account).map_or(0, |profile| {
                if current_block - profile.start_block >= profile.claim_era as u32 {
                    return 0
                }
                profile.claim_era - (current_block - profile.start_block) as u8
            })
        }

        /// Returns the profile of the given account.
        #[ink(message)]
        pub fn get_account_profile(&self, account: AccountId) -> Option<Profile> {
            self.user.get(&account)
        }

        /// Returns the profile of the caller.
        #[ink(message)]
        pub fn get_profile(&self) -> Option<Profile> {
            let caller = self.env().caller();
            self.user.get(&caller)
        }

        /// Returns the badge of the caller.
        #[ink(message)]
        pub fn get_badges(&self) -> u8 {
            self.get_profile()
                .map_or(0, |profile| profile.badges_claimed)
        }

        /// Returns the badge count of the given account.
        #[ink(message)]
        pub fn get_badges_for(&self, account: AccountId) -> u8 {
            self.get_account_profile(account)
                .map_or(0, |profile| profile.badges_claimed)
        }

        /// Mint wizard for account
        #[ink(message)]
        pub fn mint_wizard_for(&mut self, account: AccountId) -> Result<(), PSP34Error> {
            if self.get_badges_for(account) != 0 {
                // At list one badge
                match &mut self.wizard_contract {
                    Some(wizard_contract) => {
                        return wizard_contract.mint(account, Id::U8(0))
                    }
                    None => {
                        return Err(PSP34Error::Custom(String::from("NoWizardContract")))
                    }
                }
            }

            return Err(PSP34Error::Custom(String::from("NotEnoughBadges")))
        }

        /// Mint wizard for caller
        #[ink(message)]
        pub fn mint_wizard(&mut self) -> Result<(), PSP34Error> {
            let caller = self.env().caller();
            self.mint_wizard_for(caller)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn start_works() {
            let mut magink = Magink::default();
            println!("get {:?}", magink.get_remaining());
            magink.start(10);
            assert_eq!(10, magink.get_remaining());
            advance_block();
            assert_eq!(9, magink.get_remaining());
        }

        #[ink::test]
        fn claim_works() {
            const ERA: u32 = 10;
            let accounts = default_accounts();
            let mut magink = Magink::default();
            magink.start(ERA as u8);
            advance_n_blocks(ERA - 1);
            assert_eq!(1, magink.get_remaining());

            // claim fails, too early
            assert_eq!(Err(Error::TooEarlyToClaim), magink.claim());

            // claim succeeds
            advance_block();
            assert_eq!(Ok(()), magink.claim());
            assert_eq!(1, magink.get_badges());
            assert_eq!(1, magink.get_badges_for(accounts.alice));
            assert_eq!(1, magink.get_badges());
            assert_eq!(10, magink.get_remaining());

            // claim fails, too early
            assert_eq!(Err(Error::TooEarlyToClaim), magink.claim());
            advance_block();
            assert_eq!(9, magink.get_remaining());
            assert_eq!(Err(Error::TooEarlyToClaim), magink.claim());
        }

        fn default_accounts(
        ) -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<Environment>()
        }

        // fn set_sender(sender: AccountId) {
        //     ink::env::test::set_caller::<Environment>(sender);
        // }
        fn advance_n_blocks(n: u32) {
            for _ in 0..n {
                advance_block();
            }
        }
        fn advance_block() {
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        // Unfortunatly I didn't manage to get e2e-tests work -> thread 'flipper::e2e_tests::default_works' panicked at 'We should find a port before the reader ends'

        #[ink_e2e::test]
        fn new_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = MaginkRef::new();

            let _contract_acc_id = client
                .instantiate("magink", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            Ok(())
        }

        #[ink_e2e::test]
        fn mint_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = MaginkRef::new();

            let contract_acc_id = client
                .instantiate("magink", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let start = build_message::<MaginkRef>(contract_acc_id.clone())
                .call(|magink| magink.start(0));
            let _start_res = client
                .call(&ink_e2e::alice(), start, 0, None)
                .await
                .expect("start failed");

            let claim = build_message::<MaginkRef>(contract_acc_id.clone())
                .call(|magink| magink.claim());
            let _claim_res = client
                .call(&ink_e2e::alice(), claim, 0, None)
                .await
                .expect("claim failed");

            Ok(())
        }
    }
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into())
        }
    }};
}
