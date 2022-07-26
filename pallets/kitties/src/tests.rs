use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use super::*;

#[test]
fn create_kitty_works_for_default_value() {
	new_test_ext().execute_with(|| {
		System::set_block_number(10);

		let address = 1;

		log::warn!("MaxOwned 1: {:?}", <Test as Config>::MaxOwned::get());
		log::info!("count: {:?}", KittyCount::<Test>::get());
		log::error!("count own: {:?}", KittiesOwned::<Test>::get(address).len());

		assert_ok!(KittyModule::create_kitty(Origin::signed(1), 0));

		assert_eq!(KittyCount::<Test>::get(), 1);
		assert_eq!(KittiesOwned::<Test>::iter_keys().count(), 1);
		assert_eq!(KittiesOwned::<Test>::get(address).len(), 1);

		// create another kitty
		System::set_block_number(11);
		assert_ok!(KittyModule::create_kitty(Origin::signed(1), 1));

		assert_eq!(KittyCount::<Test>::get(), 2);
		assert_eq!(KittiesOwned::<Test>::iter_keys().count(), 1,  "check create second kitty - kitty owned total length failed!");
		assert_eq!(KittiesOwned::<Test>::get(address).len(), 2, "check create second kitty - kitty owned length failed!");
	});
}

#[test]
fn correct_error_for_exceed_max_kitty() {
	new_test_ext().execute_with(|| {
		System::set_block_number(10);

		let max_kitty = <Test as Config>::MaxOwned::get();

		let mut count = 0;

		while count < max_kitty {
			count = count + 1;
			KittyModule::create_kitty(Origin::signed(1), 1).unwrap();
		}
		assert_noop!(KittyModule::create_kitty(Origin::signed(1), 1), Error::<Test>::ExceedMaxKitty);
	});
}


fn get_kitty(origin: Origin, address: u64) -> Result<Kitty<Test>, ()> {
	System::set_block_number(10);

	if (KittiesOwned::<Test>::get(address).len() as u32) == 0 {
		KittyModule::create_kitty(origin, 0).unwrap();
	}
	let kitty_dna = KittyModule::kitties_owned(address)[0];

	let kitty_option = KittyModule::kitties(kitty_dna);
	match kitty_option {
		Some(kitty) => Ok(kitty),
		None => Err(()),
	}

}

#[test]
fn transfer_kitty_works_for_default_value() {
	new_test_ext().execute_with(|| {
		let from = Origin::signed(1);
		let from_address = 1;
		let to_address = 2;

		// create kitty if it's not exist
		let transfer_kitty = get_kitty(from.clone(), from_address).unwrap();

		let old_kitty_count = KittyCount::<Test>::get()  as u32;
		let old_kitties_len = Kitties::<Test>::iter_keys().count()  as u32;
		let old_kitties_owned_from_len = KittiesOwned::<Test>::get(from_address).len()  as u32;
		let old_kitties_owned_to_len = KittiesOwned::<Test>::get(to_address).len()  as u32;


		assert_ok!(KittyModule::transfer(from, to_address, transfer_kitty.dna));

		let new_kitty_count = KittyCount::<Test>::get()  as u32;
		let new_kitties_len = Kitties::<Test>::iter_keys().count()  as u32;
		let new_kitties_owned_from_len = KittiesOwned::<Test>::get(from_address).len()  as u32;
		let new_kitties_owned_to_len = KittiesOwned::<Test>::get(to_address).len()  as u32;

		assert_eq!(old_kitty_count, new_kitty_count, "check kitty count failed!");
		assert_eq!(old_kitties_len, new_kitties_len, "check kitty length failed!");
		assert_eq!(old_kitties_owned_from_len - 1, new_kitties_owned_from_len, "check owned from length failed!");
		assert_eq!(old_kitties_owned_to_len + 1, new_kitties_owned_to_len, "check owned to length failed!");
	});
}

#[test]
fn correct_error_for_transfer_invalid_kitty_owner() {
	new_test_ext().execute_with(|| {
		let from = Origin::signed(1);
		let to = Origin::signed(2);
		let to_address = 2;

		let transfer_kitty = get_kitty(to.clone(), to_address).unwrap();
		assert_noop!(KittyModule::transfer(from, to_address, transfer_kitty.dna), Error::<Test>::NotKittyOwner);
	});
}

#[test]
fn correct_error_for_transfer_invalid_transfer_to_yourself() {
	new_test_ext().execute_with(|| {
		let from = Origin::signed(1);
		let from_address = 1;

		let transfer_kitty = get_kitty(from.clone(), from_address).unwrap();

		assert_noop!(KittyModule::transfer(from, from_address, transfer_kitty.dna), Error::<Test>::TransferToYourself);
	});
}

#[test]
fn correct_error_for_transfer_to_exceed_max_kitty() {
	new_test_ext().execute_with(|| {
		let from = Origin::signed(1);
		let from_address = 1;
		let to = Origin::signed(2);
		let to_address = 2;

		// create max kitty for from address
		let max_kitty = <Test as Config>::MaxOwned::get();
		let mut count = 0;
		while count < max_kitty {
			count = count + 1;
			System::set_block_number(count.into());
			KittyModule::create_kitty(to.clone(), 1).unwrap();
		}

		let transfer_kitty = get_kitty(from.clone(), from_address).unwrap();

		assert_noop!(KittyModule::transfer(from, to_address, transfer_kitty.dna), Error::<Test>::ExceedMaxKitty);
	});
}

// Lưu ý: randomness tạo cùng 1 hash -> lỗi cần kiểm tra lại
