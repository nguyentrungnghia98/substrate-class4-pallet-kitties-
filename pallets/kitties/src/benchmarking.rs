//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Kitty;
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use frame_system::RawOrigin;

const SEED: u32 = 0;

benchmarks! {
	create_kitty {
		let price = u32::MAX;
		let caller: T::AccountId = whitelisted_caller();
	}: create_kitty(RawOrigin::Signed(caller), price)
	verify {
		assert_eq!(KittyCount::<T>::get(), 1);
	}

	transfer {
		let price = u32::MAX;
		let caller: T::AccountId = whitelisted_caller();
		let origin = <T as frame_system::Config>::Origin::from(RawOrigin::Signed(caller.clone()));

		Kitty::<T>::create_kitty(origin, 1).unwrap();

		let kitty_dna = Kitty::<T>::kitties_owned(caller.clone())[0];

		let to_address: T::AccountId = account("to_address", 0, SEED);
	}: transfer(RawOrigin::Signed(caller), to_address.clone(), kitty_dna.clone())
	verify {
		assert!(Kitty::<T>::kitties_owned(to_address).contains(&kitty_dna));
	}

	impl_benchmark_test_suite!(Kitty, crate::mock::new_test_ext(), crate::mock::Test);
}
