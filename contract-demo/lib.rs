#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;
use ink_prelude::vec::Vec;

#[ink::chain_extension]
pub trait NFTMart {
    type ErrorCode = NFTMartErr;

    #[ink(extension = 2001, returns_result = false)]
    fn fetch_random() -> [u8; 32];

    #[ink(extension = 2002, returns_result = false)]
    fn create_class(metadata: Vec<u8>, name: Vec<u8>,
                    description: Vec<u8>, properties: u8) -> (ink_env::AccountId, u32);

    #[ink(extension = 2003, returns_result = false)]
    fn proxy_mint(to: &ink_env::AccountId, class_id: u32, metadata: Vec<u8>,
                  quantity: u64, charge_royalty: Option<bool>,
    ) -> (ink_env::AccountId, ink_env::AccountId, u32, u64, u64);

	#[ink(extension = 2004, returns_result = false)]
	fn transfer(to: &ink_env::AccountId, class_id: u32, token_id: u64, quantity: u64) -> ();
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum NFTMartErr {
    Fail,
}

impl ink_env::chain_extension::FromStatusCode for NFTMartErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::Fail),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;
    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
    // type RentFraction = <ink_env::DefaultEnvironment as Environment>::RentFraction;
    type ChainExtension = NFTMart;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod contract_demo {
    use super::NFTMartErr;
    use ink_prelude::vec::Vec;

	#[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    pub struct ContractDemo {
        value: [u8; 32],
    }

    #[ink(event)]
    pub struct RandomUpdated {
        #[ink(topic)]
        new: [u8; 32],
    }

    #[ink(event)]
    pub struct CreateClassFromContract {
        #[ink(topic)]
        owner: AccountId,
        class_id: u32,
    }

    impl ContractDemo {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                value: Default::default(),
            }
        }

        #[ink(message)]
        pub fn create_class(&mut self, metadata: Vec<u8>, name: Vec<u8>, description: Vec<u8>, properties: u8) -> Result<(), NFTMartErr> {
            let (owner, class_id) = self.env().extension().create_class(metadata, name, description, properties)?;
            self.env().emit_event(CreateClassFromContract { owner, class_id });
            Ok(())
        }

        #[ink(message)]
        pub fn mint_nft(&mut self, class_id: u32, metadata: Vec<u8>,
                        quantity: u64, charge_royalty: Option<bool>,
        ) -> Result<(), NFTMartErr> {
            let (_class_owner, _beneficiary, _class_id, _token_id, _quantity) = self.env().extension().proxy_mint(
                &self.env().caller(), class_id, metadata, quantity, charge_royalty
            )?;
            Ok(())
        }

		#[ink(message)]
		pub fn transfer(&mut self, to: AccountId, class_id: u32, token_id: u64, quantity: u64) -> Result<(), NFTMartErr> {
			self.env().extension().transfer(&to, class_id, token_id, quantity)?;
			Ok(())
		}

		#[ink(message)]
		pub fn transfer_all(&mut self, to: AccountId, items: Vec<(u32, u64, u64)>) -> Result<(), NFTMartErr> {
			for (class_id, token_id, quantity) in items {
				self.env().extension().transfer(&to, class_id, token_id, quantity)?;
			}
			Ok(())
		}

        #[ink(message)]
        pub fn update(&mut self) -> Result<(), NFTMartErr> {
            let new_random = self.env().extension().fetch_random()?;
            self.value = new_random;
            self.env().emit_event(RandomUpdated { new: new_random });
            Ok(())
        }

        #[ink(message)]
        pub fn get(&self) -> [u8; 32] {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let _contract = ContractDemo::new();
        }
    }
}

