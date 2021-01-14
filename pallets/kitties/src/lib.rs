#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, Get, Randomness, ReservableCurrency},
    Parameter, StorageMap, StorageValue,
};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Bounded, Member},
    DispatchError,
};
use sp_std::vec;

// type KittyIndex = u32;

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type KittyIndex: Parameter + Member + AtLeast32BitUnsigned + Codec + Default + Copy + Bounded;
    type Reserve: Get<BalanceOf<Self>>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as Kitties {
        // Learn more about declaring storage items:
        // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;
        pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        pub OwnedKitties get(fn owned_kitties): map hasher(blake2_128_concat) T::AccountId => vec::Vec<T::KittyIndex>;
        pub KittyParents get(fn kitty_parents): map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;
        pub KittyChildren get(fn kitty_children): map hasher(blake2_128_concat) T::KittyIndex => vec::Vec<T::KittyIndex>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        KittyIndex = <T as Trait>::KittyIndex,
    {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        Created(AccountId, KittyIndex),
        Transferred(AccountId, AccountId, KittyIndex),
        Breed(AccountId, KittyIndex, KittyIndex, KittyIndex),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        KittiesCountOverflow,
        InvalidKittyId,
        RequireDifferentParent,
        NotKittyOwner,
        IndenticalAccountId,
        NotEnoughBalance,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight =0]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            T::Currency::reserve(&sender, T::Reserve::get()).map_err(|_| Error::<T>::NotEnoughBalance)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty);

            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }

        #[weight =0]
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
            let sender = ensure_signed(origin)?;

            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::NotKittyOwner)?;
            ensure!(owner == sender, Error::<T>::NotKittyOwner);
            ensure!(to != sender, Error::<T>::IndenticalAccountId);

            T::Currency::reserve(&to, T::Reserve::get()).map_err(|_| Error::<T>::NotEnoughBalance)?;
            T::Currency::unreserve(&sender, T::Reserve::get());

            <KittyOwners<T>>::insert(kitty_id, to.clone());

            if <OwnedKitties<T>>::contains_key(&owner) {
                let _ = <OwnedKitties<T>>::mutate(&owner, |val| val.retain(|&temp| temp != kitty_id));
            }

            if <OwnedKitties<T>>::contains_key(&to) {
                let _ = <OwnedKitties<T>>::mutate(&to, |val| val.push(kitty_id));
            } else {
                <OwnedKitties<T>>::insert(&to, vec![kitty_id]);
            }

            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
        }

        #[weight =0]
        pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

            Self::deposit_event(RawEvent::Breed(sender, kitty_id_1, kitty_id_2, new_kitty_id));
        }
    }
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        Kitties::<T>::insert(kitty_id, kitty);
        KittiesCount::<T>::put(kitty_id + 1.into());
        <KittyOwners<T>>::insert(kitty_id, owner);

        if <OwnedKitties<T>>::contains_key(&owner) {
            let _ = <OwnedKitties<T>>::mutate(owner, |val| val.push(kitty_id));
        } else {
            <OwnedKitties<T>>::insert(owner, vec![kitty_id]);
        }
    }

    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

    fn do_breed(
        sender: &T::AccountId,
        kitty_id_1: T::KittyIndex,
        kitty_id_2: T::KittyIndex,
    ) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        let owner = Self::kitty_owner(kitty_id_1).ok_or(Error::<T>::NotKittyOwner)?;
        ensure!(owner == *sender, Error::<T>::NotKittyOwner);
        T::Currency::reserve(&sender, T::Reserve::get())
            .map_err(|_| Error::<T>::NotEnoughBalance)?;

        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }

        let kitty_id = Self::next_kitty_id()?;
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

        if <OwnedKitties<T>>::contains_key(&sender) {
            let _ = <OwnedKitties<T>>::mutate(sender, |val| val.push(kitty_id));
        } else {
            <OwnedKitties<T>>::insert(sender, vec![kitty_id]);
        }

        <KittyParents<T>>::insert(kitty_id, vec![kitty_id_1, kitty_id_2]);
        <KittyChildren<T>>::insert(kitty_id_1, vec![kitty_id]);
        <KittyChildren<T>>::insert(kitty_id_2, vec![kitty_id]);

        Ok(kitty_id)
    }
}
