#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::{fmt::Debug};
use codec::{Codec, Encode, Decode};
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure, dispatch, 
    StorageValue, StorageMap, Parameter,
    traits::{Randomness, Get},
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
use sp_runtime::{
	RuntimeDebug,
	traits::{
		AtLeast32BitUnsigned, Member,MaybeSerializeDeserialize,
	},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Kitty<T: Trait> {
    dna: [u8;16],
    matron_id: Option<T::KittyIndex>,
    sire_id: Option<T::KittyIndex>,
    generation: u32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData<Balance> {
	free: Balance,
	reserved: Balance,
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type KittyIndex: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy + MaybeSerializeDeserialize + Debug;
    type KittyIndexUnit:  Get<Self::KittyIndex>;
    type KittyIndexMaxValue: Get<Self::KittyIndex>;
    type Balance: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy + MaybeSerializeDeserialize + Debug;
	type ExistentialDeposit: Get<Self::Balance>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
        //储存猫咪的dna数据，包括父母的kitty index（只有零代猫没有parents kitty）
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty<T>>;
        //储存猫咪的世代关系，映射关系为“parents_index=>Vec[child_index]=>Vec[child_index]”
        pub KittiesGeneration get(fn kitties_generation):
            double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => T::KittyIndex;
        //猫咪总数，每次生产猫咪后递增1个单位
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        //通过kittyindex找到owner
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        //通过owner找到所有属于这个owner的kitties
        pub OwnerKitties get(fn owner_kitties):
            double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::KittyIndex => T::KittyIndex;
        //管理create&breed kitties时的质押、转移与交易    
        pub Account: map hasher(blake2_128_concat) T::AccountId => Option<AccountData<T::Balance>>;
    }
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
        KittyIndex = <T as Trait>::KittyIndex,
        Balance = <T as Trait>::Balance
    {
        Created(AccountId, KittyIndex, Balance, Balance),
        Transferred(AccountId, AccountId, KittyIndex, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverflow,
        InvalidKittyId,
        InvalidMatronId,
        InvalidSireId,
        RequireDifferentParents,
        NotKittiesOwner,
        NotEnoughDeposit,
        NotEnoughTransferDeposit,
        NotEnoughReservedDeposit,
        NotEnoughForTransfer,
        InvalidAccount,
        TransferSelf
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        const KittyIndexUnit: T::KittyIndex = T::KittyIndexUnit::get();
        const KittyIndexMaxValue: T::KittyIndex = T::KittyIndexMaxValue::get();
        const ExistentialDeposit: T::Balance = T::ExistentialDeposit::get();

        ///creat a kitty不需要父母，create出来的都是零代猫
		#[weight = 0]
		pub fn create(
            origin,
            #[compact] new_free: T::Balance,
            #[compact] new_reserved: T::Balance
        ) -> dispatch::DispatchResult  {
            let sender = ensure_signed(origin)?;
            let kitty_id  = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty {
                dna, 
                matron_id: None, 
                sire_id: None,
                generation: 0
            };
            let existential_deposit = T::ExistentialDeposit::get();
            ensure!(new_reserved >= existential_deposit, Error::<T>::NotEnoughDeposit);
            Self::insert_kitty(&sender, kitty_id, kitty)?;
            Self::deposit(&sender, new_free, new_reserved);
            Self::deposit_event(RawEvent::Created(sender, kitty_id, new_free, new_reserved));
            Ok(())
        }
        
        #[weight = 0]
		pub fn transfer(
            origin, 
            to: T::AccountId, 
            #[compact] kitty_id: T::KittyIndex,
            #[compact] value: T::Balance
        ) -> dispatch::DispatchResult {
            
            let sender = ensure_signed(origin)?;
            ensure!(sender != to, Error::<T>::TransferSelf);
            let existential_deposit = T::ExistentialDeposit::get();
            ensure!(value >= existential_deposit, Error::<T>::NotEnoughTransferDeposit);
            let mut sender_account: AccountData::<T::Balance> = Account::<T>::get(&sender).ok_or(Error::<T>::InvalidAccount)?;
            ensure!(sender_account.reserved >= existential_deposit, Error::<T>::NotEnoughReservedDeposit);
            ensure!(sender_account.reserved >= value, Error::<T>::NotEnoughForTransfer);

            // ensure!(KittyOwners::<T>::contains_key(&kitty_id), Error::<T>::InvalidKittyId);
            let owner = KittyOwners::<T>::get(&kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            //视频中的BUG是没有验证owner关系，任何人可以随意transfer
            ensure!(sender == owner, Error::<T>::NotKittiesOwner);

            <KittyOwners<T>>::insert(kitty_id, to.clone());
            <OwnerKitties<T>>::remove(owner, kitty_id);
            <OwnerKitties<T>>::insert(to.clone(), kitty_id, kitty_id);
            Self::transfer_deposit(&sender, &mut sender_account, &to, value)?;
            
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id, value));
            Ok(())
        }
        
        //需要parents来breed child kitty，本例使用sire-kitty来计算child kitty的世代
        #[weight = 0]
		pub fn breed (
            origin, 
            #[compact] matron_kitty_id: T::KittyIndex, 
            #[compact] sire_kitty_id: T::KittyIndex,
            #[compact] new_free: T::Balance,
            #[compact] new_reserved: T::Balance
        ) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;
            let existential_deposit = T::ExistentialDeposit::get();
            ensure!(new_reserved >= existential_deposit, Error::<T>::NotEnoughDeposit);
            let new_kitty_id = Self::do_breed(&sender, matron_kitty_id, sire_kitty_id)?;
            Self::deposit(&sender, new_free, new_reserved);
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id, new_free, new_reserved));
            Ok(())
		}
	}
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) ->u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty<T>) -> dispatch::DispatchResult{
        <Kitties<T>>::insert(kitty_id,kitty.clone());
        //这里不记录零代猫，只记录次代猫
        if kitty.generation > 0 {
            <KittiesGeneration<T>>::insert(kitty.matron_id.ok_or(Error::<T>::InvalidMatronId)?, kitty_id, kitty_id);
            <KittiesGeneration<T>>::insert(kitty.sire_id.ok_or(Error::<T>::InvalidSireId)?, kitty_id, kitty_id);
        }
        <KittiesCount<T>>::put(kitty_id + T::KittyIndexUnit::get() as T::KittyIndex);
        <KittyOwners<T>>::insert(kitty_id, owner);
        <OwnerKitties<T>>::insert(owner, kitty_id, kitty_id);
        Ok(())
    }

    fn next_kitty_id() -> Result<T::KittyIndex, dispatch::DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndexMaxValue::get() as T::KittyIndex {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8;16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

    fn do_breed(sender: &T::AccountId, matron_kitty_id: T::KittyIndex, sire_kitty_id: T::KittyIndex) -> Result<T::KittyIndex,dispatch::DispatchError> {
        ensure!(matron_kitty_id != sire_kitty_id, Error::<T>::RequireDifferentParents);
        let matron_kitty: Kitty<T> = <Kitties<T>>::get(matron_kitty_id).ok_or(Error::<T>::InvalidMatronId)?;
        let sire_kitty: Kitty<T> = <Kitties<T>>::get(sire_kitty_id).ok_or(Error::<T>::InvalidSireId)?;
        let kitty_id = Self::next_kitty_id()?;
        let matron_kitty_dna = matron_kitty.dna;
        let sire_kitty_dna = sire_kitty.dna;
        let selector = Self::random_value(&sender);
        let mut dna = [0u8; 16];

        for i in 0.. matron_kitty_dna.len() {
            dna[i] = combine_dna(matron_kitty_dna[i], sire_kitty_dna[i], selector[i]);
        }
        let child_kitty = Kitty::<T> {
            dna, 
            matron_id: Some(matron_kitty_id),
            sire_id: Some(sire_kitty_id),
            generation: sire_kitty.generation + 1 //我们使用sire-kitty来计算下一代的代数
        };
        Self::insert_kitty(sender, kitty_id, child_kitty)?;
        Ok(kitty_id)
    }

    fn deposit(who: &T::AccountId, free: T::Balance, reserved: T::Balance) {
        let account = AccountData::<T::Balance> {
            free,
            reserved,
        };
        <Account<T>>::insert(who, account);
    }

    fn transfer_deposit(sender: &T::AccountId, sender_account: &mut AccountData::<T::Balance>, to: &T::AccountId, value: T::Balance)-> dispatch::DispatchResult {
        let sender_account = AccountData::<T::Balance> {
            reserved: sender_account.reserved - value,
            ..*sender_account
        };
        <Account<T>>::insert(sender, sender_account);

        let account: Option<AccountData::<T::Balance>> = Account::<T>::get(&to);
        let to_account: AccountData::<T::Balance> = match account {
            Some(to_account) => {
                AccountData::<T::Balance> {
                    free: to_account.free,
                    reserved: to_account.reserved + value,
                }
            },
            None => {
                AccountData::<T::Balance> {
                    free: 0.into(),
                    reserved: value,
                }
            }
        };
        <Account<T>>::insert(to, to_account);
        Ok(())
    }
}


