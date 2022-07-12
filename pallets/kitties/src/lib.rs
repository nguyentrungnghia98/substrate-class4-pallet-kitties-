#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
  use frame_support::inherent::Vec;

	pub type KittyDna = Vec<u8>;
	pub type KittyPrice = u32;

	#[derive(Encode, Decode, Default, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: KittyDna,
		owner: T::AccountId,
		price: KittyPrice,
		gender: Gender,
	}

	#[derive(Encode, Decode, TypeInfo)]
	pub enum Gender {
		Male,
		Female
	}

	impl Default for Gender {
		fn default() -> Self {
			Self::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitty_count)]
	pub type KittyCount<T> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Twox64Concat, KittyDna, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub type KittiesOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<KittyDna>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyCreated(T::AccountId, KittyDna),
		KittyTransferred(T::AccountId, T::AccountId, KittyDna)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		KittyCountOverflow,
		KittyNoExist,
		TransferToYourself,
		DnaAlreadyExist,
		NotKittyOwner
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// + User có thể tạo kitty
		// + Có thể chuyển đổi owner của kitty này sang một chủ khác
		// + Giới tính dựa vào độ dài của dna

		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>, dna: KittyDna, price: KittyPrice) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::kitties(&dna).is_none(), <Error<T>>::DnaAlreadyExist);

			Self::mint(&who, &dna, price);

			Self::deposit_event(Event::KittyCreated(who, dna));

			Ok(())
		}

		#[pallet::weight(100)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, dna: KittyDna) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_kitty_owner(&who, &dna)?, <Error<T>>::NotKittyOwner);

			ensure!(who != to, <Error<T>>::TransferToYourself);

			Self::transfer_kitty_to(&to, &dna);

			Self::deposit_event(Event::KittyTransferred(who, to, dna));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_gender(dna: &KittyDna) -> Gender {
			if dna.len() % 2 == 0 {
				return Gender::Male
			}

			Gender::Female
		}

		pub fn is_kitty_owner(owner: &T::AccountId, dna: &KittyDna) -> Result<bool, Error<T>> {
			match Self::kitties(dna) {
				Some(kitty) => Ok(kitty.owner == *owner),
				None => Err(<Error<T>>::KittyNoExist),
			}
		}

		pub fn mint(owner: &T::AccountId, dna: &KittyDna, price: KittyPrice) -> Result<KittyDna, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.clone(),
				gender: Self::get_gender(dna),
				owner: owner.clone(),
				price: price
			};

			let new_count = Self::kitty_count().checked_add(1).ok_or(<Error<T>>::KittyCountOverflow)?;

			<KittiesOwned<T>>::mutate(owner, |vec| vec.push(dna.clone()));

			<Kitties<T>>::insert(dna, kitty);

			<KittyCount<T>>::put(new_count);

			Ok(dna.clone())
		}

		pub fn transfer_kitty_to(to: &T::AccountId, dna: &KittyDna) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(dna).ok_or(<Error<T>>::KittyNoExist)?;

			let prev_owner = kitty.owner.clone();

			<KittiesOwned<T>>::try_mutate(&prev_owner, |vec| {
				if let Some(pos) = vec.iter().position(|x| *x == *dna) {
					vec.swap_remove(pos);
					return Ok(());
				}

				Err(())
			}).map_err(|_| <Error<T>>::KittyNoExist);

			kitty.owner = to.clone();

			<Kitties<T>>::insert(dna, kitty);

			<KittiesOwned<T>>::mutate(to, |vec| vec.push(dna.clone()));

			Ok(())
		}
	}
}
