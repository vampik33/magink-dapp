#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

pub use self::wizard::WizardRef;

#[openbrush::implementation(PSP34, Ownable, PSP34Metadata, PSP34Mintable)]
#[openbrush::contract]
pub mod wizard {
    use ink::codegen::EmitEvent;
    use ink::codegen::Env;
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Wizard {
        #[storage_field]
        psp34: psp34::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        token_id: Id,
    }

    #[overrider(PSP34Mintable)]
    #[openbrush::modifiers(only_owner)]
    fn mint(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
        let result = psp34::InternalImpl::_mint_to(self, account, id.clone());
        if result.is_ok() {
            self.env().emit_event(Transfer {
                from: None,
                to: Some(account),
                token_id: id,
            });
        }
        result
    }

    impl Wizard {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut _instance = Self::default();
            ownable::Internal::_init_with_owner(&mut _instance, Self::env().caller());
            psp34::Internal::_mint_to(&mut _instance, Self::env().caller(), Id::U8(1))
                .expect("Can mint");
            let collection_id = psp34::PSP34Impl::collection_id(&_instance);
            metadata::Internal::_set_attribute(
                &mut _instance,
                collection_id.clone(),
                String::from("name"),
                String::from("Wizard"),
            );
            metadata::Internal::_set_attribute(
                &mut _instance,
                collection_id.clone(),
                String::from("symbol"),
                String::from("WZRD"),
            );
            metadata::Internal::_set_attribute(
                &mut _instance,
                collection_id,
                String::from("image"),
                String::from("QmcHvJ5gPTEHSS8aNW9sTuxyJJynmoMb1j1FkKFRZHSNwy"), // Don't like this, hardcoded image from IPFS
            );
            _instance
        }
    }
}
