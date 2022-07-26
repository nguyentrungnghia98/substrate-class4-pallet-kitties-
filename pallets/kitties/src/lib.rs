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
	use frame_support::traits::{ Randomness, Time, Get };
	use frame_support::dispatch::fmt;
	use frame_system::pallet_prelude::*;
	// use frame_support::inherent::Vec;

	pub type KittyDna<T> = <T as frame_system::Config>::Hash;
	pub type KittyPrice = u32;
	pub type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

	#[derive(Encode, Decode, Default, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: KittyDna<T>,
		pub owner: T::AccountId,
		pub price: KittyPrice,
		pub gender: Gender,
		pub created_date: MomentOf<T>
	}

	impl<T: Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
			 .field("dna", &self.dna)
			 .field("owner", &self.owner)
			 .field("price", &self.price)
			 .field("gender", &self.gender)
			 .field("created_date", &self.created_date)
			 .finish()
		}
	}

	#[derive(Encode, Decode, TypeInfo, Debug)]
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

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type Time: Time;

		type MaxOwned: Get<u32>;
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
	pub type Kitties<T: Config> = StorageMap<_, Twox64Concat, KittyDna<T>, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub type KittiesOwned<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<KittyDna<T>, T::MaxOwned>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyCreated(T::AccountId, KittyDna<T>),
		KittyTransferred(T::AccountId, T::AccountId, KittyDna<T>),
		TestHash(u8)
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
		NotKittyOwner,
		ExceedMaxKitty,
		PriceMustGreaterThanZero
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(363_600_000 + T::DbWeight::get().reads_writes(4,3))]
		pub fn create_kitty(origin: OriginFor<T>, price: KittyPrice) -> DispatchResult {
			let who = ensure_signed(origin)?;

			log::info!("MaxOwned: {:?}", T::MaxOwned::get());
			log::warn!("MaxOwned: {:?}", T::MaxOwned::get());
			log::error!("MaxOwned: {:?}", T::MaxOwned::get());

			ensure!(!Self::is_exceed_max_kitty(&who), <Error<T>>::ExceedMaxKitty);

			ensure!(price >= 0, <Error<T>>::PriceMustGreaterThanZero);

			let dna = Self::mint(&who, price)?;

			Self::deposit_event(Event::KittyCreated(who, dna));

			Ok(())
		}

		#[pallet::weight(123_200_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, dna: KittyDna<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_kitty_owner(&who, &dna)?, <Error<T>>::NotKittyOwner);

			ensure!(who != to, <Error<T>>::TransferToYourself);

			ensure!(!Self::is_exceed_max_kitty(&to), <Error<T>>::ExceedMaxKitty);

			Self::transfer_kitty_to(&to, &dna)?;

			Self::deposit_event(Event::KittyTransferred(who, to, dna));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn generate_dna() -> T::Hash {
			T::Randomness::random(&b"dna"[..]).0
		}

		pub fn is_exceed_max_kitty(owner: &T::AccountId) -> bool {
			let kitties = Self::kitties_owned(owner);

			(kitties.len() as u32) >= T::MaxOwned::get()
		}

		pub fn get_gender(dna: &KittyDna<T>) -> Gender {
			if dna.as_ref()[0] % 2 == 0 {
				return Gender::Male
			}

			Gender::Female
		}

		pub fn is_kitty_owner(owner: &T::AccountId, dna: &KittyDna<T>) -> Result<bool, Error<T>> {
			match Self::kitties(dna) {
				Some(kitty) => Ok(kitty.owner == *owner),
				None => Err(<Error<T>>::KittyNoExist),
			}
		}

		pub fn mint(owner: &T::AccountId, price: KittyPrice) -> Result<KittyDna<T>, Error<T>> {
			let dna = Self::generate_dna();

			let kitty = Kitty::<T> {
				dna: dna,
				gender: Self::get_gender(&dna),
				owner: owner.clone(),
				price: price,
				created_date: T::Time::now()
			};

			log::info!("A kitty is born: {:?}", kitty);
			log::warn!("A kitty is born: {:?}", kitty);
			log::error!("A kitty is born: {:?}", kitty);

			let new_count = Self::kitty_count().checked_add(1).ok_or(<Error<T>>::KittyCountOverflow)?;

			<KittiesOwned<T>>::try_mutate(owner, |vec| vec.try_push(dna.clone())).map_err(|_| <Error<T>>::ExceedMaxKitty)?;

			<Kitties<T>>::insert(dna, kitty);

			<KittyCount<T>>::put(new_count);

			Ok(dna.clone())
		}

		pub fn transfer_kitty_to(to: &T::AccountId, dna: &KittyDna<T>) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(dna).ok_or(<Error<T>>::KittyNoExist)?;

			let prev_owner = kitty.owner.clone();

			<KittiesOwned<T>>::try_mutate(&prev_owner, |vec| {
				if let Some(pos) = vec.iter().position(|x| *x == *dna) {
					vec.swap_remove(pos);
					return Ok(());
				}

				Err(())
			}).map_err(|_| <Error<T>>::KittyNoExist)?;

			kitty.owner = to.clone();

			<Kitties<T>>::insert(dna, kitty);

			<KittiesOwned<T>>::try_mutate(to, |vec| vec.try_push(dna.clone())).map_err(|_| <Error<T>>::ExceedMaxKitty)?;

			Ok(())
		}
	}
}
