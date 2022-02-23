//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use mock::{Event, *};
use sp_runtime::{traits::BadOrigin, TokenError};

// *************************************************
// tests for genesis
// *************************************************

#[test]
fn genesis_issuance_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 0);
			assert_eq!(Tokens::total_issuance(AXC), 200);
		});
}

// *************************************************
// tests for call
// *************************************************

#[test]
fn transfer_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::transfer(Some(ALICE).into(), BOB, AXC, 50));
			System::assert_last_event(Event::Tokens(crate::Event::Transfer(AXC, ALICE, BOB, 50)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 150);
			assert_eq!(Tokens::total_issuance(AXC), 200);

			assert_noop!(
				Tokens::transfer(Some(ALICE).into(), BOB, AXC, 60),
				Error::<Runtime>::BalanceTooLow,
			);
			assert_noop!(
				Tokens::transfer(Some(ALICE).into(), CHARLIE, AXC, 1),
				Error::<Runtime>::ExistentialDeposit,
			);
			assert_ok!(Tokens::transfer(Some(ALICE).into(), CHARLIE, AXC, 2));

			// imply AllowDeath
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_ok!(Tokens::transfer(Some(ALICE).into(), BOB, AXC, 48));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 198);
			assert_eq!(Tokens::total_issuance(AXC), 200);
		});
}

#[test]
fn transfer_keep_alive_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);

			// imply KeepAlive
			assert_noop!(
				Tokens::transfer_keep_alive(Some(ALICE).into(), BOB, AXC, 99),
				Error::<Runtime>::KeepAlive,
			);

			assert_ok!(Tokens::transfer_keep_alive(Some(ALICE).into(), BOB, AXC, 98));
			System::assert_last_event(Event::Tokens(crate::Event::Transfer(AXC, ALICE, BOB, 98)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 2);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 198);
		});
}

#[test]
fn transfer_all_keep_alive_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_ok!(Tokens::transfer_all(Some(ALICE).into(), CHARLIE, AXC, true));
			System::assert_has_event(Event::Tokens(crate::Event::Transfer(AXC, ALICE, CHARLIE, 98)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 2);

			assert_ok!(Tokens::set_lock(ID_1, AXC, &BOB, 50));
			assert_eq!(Tokens::accounts(&BOB, AXC).frozen, 50);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_ok!(Tokens::transfer_all(Some(BOB).into(), CHARLIE, AXC, true));
			System::assert_has_event(Event::Tokens(crate::Event::Transfer(AXC, BOB, CHARLIE, 50)));
		});
}

#[test]
fn transfer_all_allow_death_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_ok!(Tokens::transfer_all(Some(ALICE).into(), CHARLIE, AXC, false));
			System::assert_last_event(Event::Tokens(crate::Event::Transfer(AXC, ALICE, CHARLIE, 100)));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);

			assert_ok!(Tokens::set_lock(ID_1, AXC, &BOB, 50));
			assert_eq!(Tokens::accounts(&BOB, AXC).frozen, 50);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_ok!(Tokens::transfer_all(Some(BOB).into(), CHARLIE, AXC, false));
			System::assert_last_event(Event::Tokens(crate::Event::Transfer(AXC, BOB, CHARLIE, 50)));
		});
}

#[test]
fn force_transfer_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_noop!(
				Tokens::force_transfer(Some(ALICE).into(), ALICE, BOB, AXC, 100),
				BadOrigin
			);

			// imply AllowDeath
			assert_ok!(Tokens::force_transfer(RawOrigin::Root.into(), ALICE, BOB, AXC, 100));
			System::assert_last_event(Event::Tokens(crate::Event::Transfer(AXC, ALICE, BOB, 100)));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 200);
		});
}

#[test]
fn set_balance_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			// bad origin
			assert_noop!(Tokens::set_balance(Some(ALICE).into(), ALICE, AXC, 200, 100), BadOrigin);

			// total balance overflow
			assert_noop!(
				Tokens::set_balance(RawOrigin::Root.into(), ALICE, AXC, Balance::max_value(), 1),
				ArithmeticError::Overflow
			);

			// total issurance overflow
			assert_noop!(
				Tokens::set_balance(RawOrigin::Root.into(), ALICE, AXC, Balance::max_value(), 0),
				ArithmeticError::Overflow
			);

			// total issurance overflow
			assert_noop!(
				Tokens::set_balance(RawOrigin::Root.into(), ALICE, AXC, Balance::max_value(), 0),
				ArithmeticError::Overflow
			);

			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 200);

			assert_ok!(Tokens::set_balance(RawOrigin::Root.into(), ALICE, AXC, 200, 100));
			System::assert_has_event(Event::Tokens(crate::Event::BalanceSet(AXC, ALICE, 200, 100)));
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 200);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::total_issuance(AXC), 400);

			assert!(Accounts::<Runtime>::contains_key(BOB, AXC));
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 0);

			assert_ok!(Tokens::set_balance(RawOrigin::Root.into(), BOB, AXC, 0, 0));
			System::assert_has_event(Event::Tokens(crate::Event::BalanceSet(AXC, BOB, 0, 0)));
			assert!(!Accounts::<Runtime>::contains_key(BOB, AXC));
			assert_eq!(Tokens::free_balance(AXC, &BOB), 0);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 0);
			assert_eq!(Tokens::total_issuance(AXC), 300);

			assert!(!Accounts::<Runtime>::contains_key(CHARLIE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 0);
			assert_eq!(Tokens::reserved_balance(AXC, &CHARLIE), 0);

			// below ED,
			assert_ok!(Tokens::set_balance(RawOrigin::Root.into(), CHARLIE, AXC, 1, 0));
			System::assert_has_event(Event::Tokens(crate::Event::BalanceSet(AXC, CHARLIE, 0, 0)));
			assert!(!Accounts::<Runtime>::contains_key(CHARLIE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 0);
			assert_eq!(Tokens::reserved_balance(AXC, &CHARLIE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 300);
		});
}

// *************************************************
// tests for inline impl
// *************************************************

#[test]
fn deposit_consequence_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			Tokens::deposit_consequence(
				&CHARLIE,
				AXC,
				0,
				&AccountData {
					free: 1,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Ok(())
		);

		// total issuance overflow
		assert_eq!(
			Tokens::deposit_consequence(
				&CHARLIE,
				AXC,
				Balance::max_value(),
				&AccountData {
					free: 1,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Err(ArithmeticError::Overflow.into())
		);

		// total balance overflow
		assert_eq!(
			Tokens::deposit_consequence(
				&CHARLIE,
				AXC,
				1,
				&AccountData {
					free: Balance::max_value(),
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Err(ArithmeticError::Overflow.into())
		);

		// below ed
		assert_eq!(
			Tokens::deposit_consequence(
				&CHARLIE,
				AXC,
				1,
				&AccountData {
					free: 0,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Err(TokenError::BelowMinimum.into())
		);

		assert_eq!(
			Tokens::deposit_consequence(
				&CHARLIE,
				AXC,
				1,
				&AccountData {
					free: 1,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Ok(())
		);
	});
}

#[test]
fn withdraw_consequence_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				0,
				&AccountData {
					free: 1,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Ok(Zero::zero())
		);

		// total issuance underflow
		assert_ok!(Tokens::update_balance(AXC, &ALICE, 2));
		assert_eq!(Tokens::total_issuance(AXC), 2);
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				3,
				&AccountData {
					free: 1,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Err(ArithmeticError::Underflow.into())
		);

		// total issuance is not enough
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				2,
				&AccountData {
					free: 1,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Err(TokenError::NoFunds.into())
		);

		// below ED and cannot dec provider
		assert_ok!(Tokens::update_balance(AXC, &ALICE, 2));
		assert_eq!(System::providers(&ALICE), 1);
		assert_ok!(System::inc_consumers(&ALICE));
		assert!(!System::can_dec_provider(&ALICE));
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				1,
				&AccountData {
					free: 2,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Err(TokenError::WouldDie.into())
		);

		// below ED and can dec provider
		let _ = System::inc_providers(&ALICE);
		assert!(System::can_dec_provider(&ALICE));
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				1,
				&AccountData {
					free: 2,
					reserved: 0,
					frozen: 0
				}
			)
			.into_result(),
			Ok(1)
		);

		// free balance is not enough
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				2,
				&AccountData {
					free: 1,
					reserved: 1,
					frozen: 0
				}
			)
			.into_result(),
			Err(TokenError::NoFunds.into())
		);

		// less to frozen balance
		assert_eq!(
			Tokens::withdraw_consequence(
				&ALICE,
				AXC,
				2,
				&AccountData {
					free: 2,
					reserved: 0,
					frozen: 2
				}
			)
			.into_result(),
			Err(TokenError::Frozen.into())
		);
	});
}

#[test]
fn ensure_can_withdraw_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);

			assert_noop!(
				Tokens::ensure_can_withdraw(AXC, &ALICE, 101),
				Error::<Runtime>::BalanceTooLow
			);

			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 50));
			assert_noop!(
				Tokens::ensure_can_withdraw(AXC, &ALICE, 51),
				Error::<Runtime>::LiquidityRestrictions
			);

			assert_ok!(Tokens::ensure_can_withdraw(AXC, &ALICE, 50));
		});
}

#[test]
fn set_free_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		/* Scenarios: ED is not zero, account is not in dust removal whitelist */
		assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
		assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 0);
		assert_eq!(Tokens::total_issuance(AXC), 0);

		// when total is below ED, account will be reaped.
		Tokens::set_free_balance(AXC, &ALICE, 1);
		assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
		assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);
		// set_free_balance do not change total issuance.
		assert_eq!(Tokens::total_issuance(AXC), 0);

		Tokens::set_free_balance(AXC, &ALICE, 2);
		assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &ALICE), 2);
		assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);

		/* Scenarios: ED is not zero, account is in dust removal whitelist */
		assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &DAVE), 0);
		assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);

		// set zero will not create account
		Tokens::set_free_balance(AXC, &DAVE, 0);
		assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));

		// when total is below ED, account will not be reaped.
		Tokens::set_free_balance(AXC, &DAVE, 1);
		assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &DAVE), 1);
		assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);

		/* Scenarios: ED is zero */
		assert!(!Accounts::<Runtime>::contains_key(ALICE, ETH));
		assert_eq!(Tokens::free_balance(ETH, &ALICE), 0);
		assert_eq!(Tokens::free_balance(ETH, &DustReceiver::get()), 0);

		// set zero will create account
		Tokens::set_free_balance(ETH, &ALICE, 0);
		assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
		assert_eq!(Tokens::free_balance(ETH, &ALICE), 0);
		assert_eq!(Tokens::free_balance(ETH, &DustReceiver::get()), 0);
	});
}

#[test]
fn set_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		/* Scenarios: ED is not zero, account is not in dust removal whitelist */
		assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
		assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
		assert_eq!(Tokens::total_issuance(AXC), 0);

		// when total is below ED, account should be reaped.
		Tokens::set_reserved_balance(AXC, &ALICE, 1);
		// but reap it failed because failed to transfer/withdraw dust removal!!!
		assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
		assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 1);
		// set_reserved_balance do not change total issuance.
		assert_eq!(Tokens::total_issuance(AXC), 0);

		Tokens::set_reserved_balance(AXC, &ALICE, 2);
		assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
		assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 2);

		/* Scenarios: ED is not zero, account is in dust removal whitelist */
		assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &DAVE), 0);

		// set zero will not create account
		Tokens::set_reserved_balance(AXC, &DAVE, 0);
		assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));

		// when total is below ED, account shouldn't be reaped.
		Tokens::set_reserved_balance(AXC, &DAVE, 1);
		assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
		assert_eq!(Tokens::reserved_balance(AXC, &DAVE), 1);

		/* Scenarios: ED is zero */
		assert!(!Accounts::<Runtime>::contains_key(ALICE, ETH));
		assert_eq!(Tokens::reserved_balance(ETH, &ALICE), 0);

		// set zero will create account
		Tokens::set_reserved_balance(ETH, &ALICE, 0);
		assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
		assert_eq!(Tokens::reserved_balance(ETH, &ALICE), 0);
	});
}

#[test]
fn do_transfer_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			// always ok when from == to
			assert_ok!(Tokens::do_transfer(
				AXC,
				&ALICE,
				&ALICE,
				101,
				ExistenceRequirement::KeepAlive
			));

			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 0);

			assert_noop!(
				Tokens::do_transfer(AXC, &ALICE, &BOB, 101, ExistenceRequirement::KeepAlive),
				Error::<Runtime>::BalanceTooLow
			);
			assert_noop!(
				Tokens::do_transfer(AXC, &ALICE, &CHARLIE, 1, ExistenceRequirement::KeepAlive),
				Error::<Runtime>::ExistentialDeposit
			);

			assert_ok!(Tokens::do_transfer(
				AXC,
				&ALICE,
				&BOB,
				100,
				ExistenceRequirement::AllowDeath
			));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 200);
		});
}

#[test]
fn do_transfer_dust_removal_when_allow_death() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 0);

			assert_ok!(Tokens::do_transfer(
				AXC,
				&ALICE,
				&BOB,
				99,
				ExistenceRequirement::AllowDeath
			));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 199);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);
		});
}

#[test]
fn do_transfer_report_keep_alive_error_when_ed_is_not_zero() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (DAVE, AXC, 100)])
		.build()
		.execute_with(|| {
			// total of alice will below ED, account will be reaped.
			assert_noop!(
				Tokens::do_transfer(AXC, &ALICE, &BOB, 99, ExistenceRequirement::KeepAlive),
				Error::<Runtime>::KeepAlive
			);

			// even if dave is in dust removal whitelist, but account drain will still cause
			// account be be reaped.
			assert_noop!(
				Tokens::do_transfer(AXC, &DAVE, &BOB, 100, ExistenceRequirement::KeepAlive),
				Error::<Runtime>::KeepAlive
			);

			// as long as do not transfer all balance, even if the total is below ED, the
			// account will not be reaped.
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 0);
			assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_ok!(Tokens::do_transfer(
				AXC,
				&DAVE,
				&BOB,
				99,
				ExistenceRequirement::KeepAlive
			));
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 1);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 99);
			assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
		});
}

#[test]
fn do_transfer_will_not_report_keep_alive_error_when_ed_is_zero() {
	ExtBuilder::default()
		.balances(vec![(ALICE, ETH, 100), (DAVE, ETH, 100)])
		.build()
		.execute_with(|| {
			assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &ALICE), 100);
			assert_eq!(Tokens::free_balance(ETH, &BOB), 0);
			assert_ok!(Tokens::do_transfer(
				ETH,
				&ALICE,
				&BOB,
				99,
				ExistenceRequirement::KeepAlive
			));
			assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &ALICE), 1);
			assert_eq!(Tokens::free_balance(ETH, &BOB), 99);

			// account that total is zero will not be reaped because ED is zero
			assert!(Accounts::<Runtime>::contains_key(DAVE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &DAVE), 100);
			assert_ok!(Tokens::do_transfer(
				ETH,
				&DAVE,
				&BOB,
				100,
				ExistenceRequirement::KeepAlive
			));
			assert!(Accounts::<Runtime>::contains_key(DAVE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &DAVE), 0);
			assert_eq!(Tokens::free_balance(ETH, &BOB), 199);
		});
}

#[test]
fn do_transfer_report_keep_alive_error_due_to_cannot_dec_provider_when_allow_death() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (DAVE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert!(System::can_dec_provider(&ALICE));
			assert_ok!(System::inc_consumers(&ALICE));
			assert!(!System::can_dec_provider(&ALICE));
			assert_noop!(
				Tokens::do_transfer(AXC, &ALICE, &BOB, 99, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::KeepAlive
			);

			assert_ok!(Tokens::deposit(BTC, &ALICE, 100));
			assert!(System::can_dec_provider(&ALICE));
			assert_ok!(Tokens::do_transfer(
				AXC,
				&ALICE,
				&BOB,
				99,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn do_transfer_report_existential_deposit_error_when_ed_is_not_zero() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Tokens::do_transfer(AXC, &ALICE, &BOB, 1, ExistenceRequirement::KeepAlive),
				Error::<Runtime>::ExistentialDeposit
			);

			// if receiver is in dust removal whitelist, will not be reaped.
			assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 0);
			assert_ok!(Tokens::do_transfer(
				AXC,
				&ALICE,
				&DAVE,
				1,
				ExistenceRequirement::KeepAlive
			));
			assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 1);
		});
}

#[test]
fn do_withdraw_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			// always ok if amount is zero
			assert!(!Accounts::<Runtime>::contains_key(BOB, AXC));
			assert_ok!(Tokens::do_withdraw(AXC, &BOB, 0, ExistenceRequirement::KeepAlive, true));
			assert!(!Accounts::<Runtime>::contains_key(BOB, AXC));

			assert_noop!(
				Tokens::do_withdraw(AXC, &ALICE, 101, ExistenceRequirement::KeepAlive, true),
				Error::<Runtime>::BalanceTooLow
			);

			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 10));
			assert_noop!(
				Tokens::do_withdraw(AXC, &ALICE, 91, ExistenceRequirement::KeepAlive, true),
				Error::<Runtime>::LiquidityRestrictions
			);

			// change issuance
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_ok!(Tokens::do_withdraw(
				AXC,
				&ALICE,
				10,
				ExistenceRequirement::KeepAlive,
				true
			));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 90);
			assert_eq!(Tokens::total_issuance(AXC), 90);

			// do not change issuance
			assert_ok!(Tokens::do_withdraw(
				AXC,
				&ALICE,
				10,
				ExistenceRequirement::KeepAlive,
				false
			));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 80);
			assert_eq!(Tokens::total_issuance(AXC), 90);
		});
}

#[test]
fn do_withdraw_dust_removal_when_allow_death() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 0);

			assert_ok!(Tokens::do_withdraw(
				AXC,
				&ALICE,
				99,
				ExistenceRequirement::AllowDeath,
				true
			));
			assert_eq!(Tokens::total_issuance(AXC), 1);
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);
		});
}

#[test]
fn do_withdraw_report_keep_alive_error_when_ed_is_not_zero() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (DAVE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Tokens::do_withdraw(AXC, &ALICE, 99, ExistenceRequirement::KeepAlive, true),
				Error::<Runtime>::KeepAlive
			);

			// dave is in dust removal whitelist, still can withdraw if remainer is not zero
			// but below ED.
			assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 100);
			assert_eq!(Tokens::total_issuance(AXC), 200);
			assert_ok!(Tokens::do_withdraw(
				AXC,
				&DAVE,
				99,
				ExistenceRequirement::KeepAlive,
				true
			));
			assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 1);
			assert_eq!(Tokens::total_issuance(AXC), 101);

			// even if dave is in dust removal whitelist, but if withdraw all total of it
			// will still cause account reaped.
			assert_noop!(
				Tokens::do_withdraw(AXC, &DAVE, 1, ExistenceRequirement::KeepAlive, true),
				Error::<Runtime>::KeepAlive
			);
		});
}

#[test]
fn do_withdraw_will_not_report_keep_alive_error_when_ed_is_zero() {
	ExtBuilder::default()
		.balances(vec![(ALICE, ETH, 100), (DAVE, ETH, 100)])
		.build()
		.execute_with(|| {
			assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &ALICE), 100);
			assert_eq!(Tokens::total_issuance(ETH), 200);
			assert_ok!(Tokens::do_withdraw(
				ETH,
				&ALICE,
				100,
				ExistenceRequirement::KeepAlive,
				true
			));
			assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(ETH), 100);

			assert!(Accounts::<Runtime>::contains_key(DAVE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &DAVE), 100);
			assert_ok!(Tokens::do_withdraw(
				ETH,
				&DAVE,
				100,
				ExistenceRequirement::KeepAlive,
				true
			));
			assert!(Accounts::<Runtime>::contains_key(DAVE, ETH));
			assert_eq!(Tokens::free_balance(ETH, &DAVE), 0);
			assert_eq!(Tokens::total_issuance(ETH), 0);
		});
}

#[test]
fn do_deposit_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			// always ok if deposit amount is zero
			assert_ok!(Tokens::do_deposit(AXC, &CHARLIE, 0, true, true));
			assert_ok!(Tokens::do_deposit(AXC, &CHARLIE, 0, false, true));

			assert!(!Accounts::<Runtime>::contains_key(CHARLIE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_ok!(Tokens::do_deposit(AXC, &CHARLIE, 10, false, true));
			assert!(Accounts::<Runtime>::contains_key(CHARLIE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 10);
			assert_eq!(Tokens::total_issuance(AXC), 110);

			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_ok!(Tokens::do_deposit(AXC, &ALICE, 10, true, true));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 110);
			assert_eq!(Tokens::total_issuance(AXC), 120);

			assert_noop!(
				Tokens::do_deposit(AXC, &ALICE, Balance::max_value(), false, true),
				ArithmeticError::Overflow
			);

			// do not change issuance
			assert_ok!(Tokens::do_deposit(AXC, &ALICE, 100, true, false));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 210);
			assert_eq!(Tokens::total_issuance(AXC), 120);
		});
}

#[test]
fn do_deposit_report_dead_account_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!Accounts::<Runtime>::contains_key(CHARLIE, AXC));
		assert_noop!(
			Tokens::do_deposit(AXC, &CHARLIE, 10, true, true),
			Error::<Runtime>::DeadAccount
		);
	});
}

#[test]
fn do_deposit_report_existential_deposit_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!Accounts::<Runtime>::contains_key(CHARLIE, AXC));
		assert_noop!(
			Tokens::do_deposit(AXC, &CHARLIE, 1, false, true),
			Error::<Runtime>::ExistentialDeposit
		);

		assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &DAVE), 0);
		assert_eq!(Tokens::total_issuance(AXC), 0);
		assert_ok!(Tokens::do_deposit(AXC, &DAVE, 1, false, true));
		assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &DAVE), 1);
		assert_eq!(Tokens::total_issuance(AXC), 1);
	});
}

// *************************************************
// tests for endowed account and remove account
// *************************************************

#[test]
fn endowed_account_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(System::providers(&ALICE), 0);
		assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
		Tokens::set_free_balance(AXC, &ALICE, 100);
		System::assert_last_event(Event::Tokens(crate::Event::Endowed(AXC, ALICE, 100)));
		assert_eq!(System::providers(&ALICE), 1);
		assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
	});
}

#[test]
fn remove_account_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(System::providers(&ALICE), 1);
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			Tokens::set_free_balance(AXC, &ALICE, 0);
			assert_eq!(System::providers(&ALICE), 0);
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
		});
}

#[test]
fn reap_account_will_dec_providers_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (ALICE, ETH, 100), (ALICE, BTC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(System::providers(&ALICE), 3);
			assert!(System::account_exists(&ALICE));
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));

			assert_ok!(Tokens::do_transfer(
				AXC,
				&ALICE,
				&BOB,
				100,
				ExistenceRequirement::AllowDeath
			));
			assert_eq!(System::providers(&ALICE), 2);
			assert!(System::account_exists(&ALICE));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));

			// ED of ETH is zero, the account will retain even if the total is zero,
			// will not dec_providers
			assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));
			assert_ok!(Tokens::do_transfer(
				ETH,
				&ALICE,
				&BOB,
				100,
				ExistenceRequirement::AllowDeath
			));
			assert_eq!(System::providers(&ALICE), 2);
			assert!(System::account_exists(&ALICE));
			assert!(Accounts::<Runtime>::contains_key(ALICE, ETH));

			assert!(Accounts::<Runtime>::contains_key(ALICE, BTC));
			assert_ok!(Tokens::do_transfer(
				BTC,
				&ALICE,
				&BOB,
				100,
				ExistenceRequirement::AllowDeath
			));
			assert_eq!(System::providers(&ALICE), 1);
			assert!(System::account_exists(&ALICE));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, BTC));
		});
}

#[test]
fn dust_removal_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(System::providers(&ALICE), 1);
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 0);
			Tokens::set_free_balance(AXC, &ALICE, 1);
			System::assert_last_event(Event::Tokens(crate::Event::DustLost(AXC, ALICE, 1)));
			assert_eq!(System::providers(&ALICE), 0);
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &DustReceiver::get()), 1);

			// dave is in dust removal whitelist, will not remove its dust even if its total
			// below ED
			assert!(!Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_eq!(System::providers(&DAVE), 0);
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 0);
			Tokens::set_free_balance(AXC, &DAVE, 1);
			assert!(Accounts::<Runtime>::contains_key(DAVE, AXC));
			assert_eq!(System::providers(&DAVE), 1);
			assert_eq!(Tokens::free_balance(AXC, &DAVE), 1);
			System::assert_last_event(Event::Tokens(crate::Event::Endowed(AXC, DAVE, 1)));
		});
}

#[test]
fn account_survive_due_to_dust_transfer_failure() {
	ExtBuilder::default().build().execute_with(|| {
		let dust_account = DustReceiver::get();
		Tokens::set_free_balance(AXC, &dust_account, 0);
		assert_eq!(Tokens::free_balance(AXC, &dust_account), 0);
		assert_eq!(Tokens::total_balance(AXC, &ALICE), 0);
		assert_eq!(System::providers(&ALICE), 0);
		assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));

		Tokens::set_reserved_balance(AXC, &ALICE, 1);
		System::assert_last_event(Event::Tokens(crate::Event::DustLost(AXC, ALICE, 1)));
		assert_eq!(Tokens::free_balance(AXC, &dust_account), 0);
		assert_eq!(Tokens::total_balance(AXC, &ALICE), 1);
		assert_eq!(System::providers(&ALICE), 1);
		assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
	});
}

// *************************************************
// tests for MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency,
// MultiReservableCurrency traits **********************************************
// ***

#[test]
fn multicurrency_deposit_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!Accounts::<Runtime>::contains_key(CHARLIE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 0);
		assert_eq!(Tokens::total_issuance(AXC), 0);
		assert_ok!(Tokens::deposit(AXC, &CHARLIE, 10));
		assert!(Accounts::<Runtime>::contains_key(CHARLIE, AXC));
		assert_eq!(Tokens::free_balance(AXC, &CHARLIE), 10);
		assert_eq!(Tokens::total_issuance(AXC), 10);
	});
}

#[test]
fn multicurrency_withdraw_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_ok!(Tokens::withdraw(AXC, &ALICE, 99));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 1);
		});
}

#[test]
fn multicurrency_transfer_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert!(Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_ok!(<Tokens as MultiCurrency<_>>::transfer(AXC, &ALICE, &BOB, 99));
			assert!(!Accounts::<Runtime>::contains_key(ALICE, AXC));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 199);
		});
}

#[test]
fn multicurrency_can_slash_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert!(!Tokens::can_slash(AXC, &ALICE, 101));
			assert!(Tokens::can_slash(AXC, &ALICE, 100));
		});
}

#[test]
fn multicurrency_slash_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			// slashed_amount < amount
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_eq!(Tokens::slash(AXC, &ALICE, 50), 0);
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(AXC), 50);

			// slashed_amount == amount
			assert_eq!(Tokens::slash(AXC, &ALICE, 51), 1);
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 0);
		});
}

#[test]
fn multicurrency_extended_update_balance_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::update_balance(AXC, &ALICE, 50));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 150);
			assert_eq!(Tokens::total_issuance(AXC), 250);

			assert_ok!(Tokens::update_balance(AXC, &BOB, -50));
			assert_eq!(Tokens::free_balance(AXC, &BOB), 50);
			assert_eq!(Tokens::total_issuance(AXC), 200);

			assert_noop!(Tokens::update_balance(AXC, &BOB, -60), Error::<Runtime>::BalanceTooLow);
		});
}

#[test]
fn multi_lockable_currency_set_lock_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 10));
			assert_eq!(Tokens::accounts(&ALICE, AXC).frozen, 10);
			assert_eq!(Tokens::accounts(&ALICE, AXC).frozen(), 10);
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 1);
			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 50));
			assert_eq!(Tokens::accounts(&ALICE, AXC).frozen, 50);
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 1);
			assert_ok!(Tokens::set_lock(ID_2, AXC, &ALICE, 60));
			assert_eq!(Tokens::accounts(&ALICE, AXC).frozen, 60);
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 2);
		});
}

#[test]
fn multi_lockable_currency_extend_lock_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 10));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 1);
			assert_eq!(Tokens::accounts(&ALICE, AXC).frozen, 10);
			assert_ok!(Tokens::extend_lock(ID_1, AXC, &ALICE, 20));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 1);
			assert_eq!(Tokens::accounts(&ALICE, AXC).frozen, 20);
			assert_ok!(Tokens::extend_lock(ID_2, AXC, &ALICE, 10));
			assert_ok!(Tokens::extend_lock(ID_1, AXC, &ALICE, 20));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 2);
		});
}

#[test]
fn multi_lockable_currency_remove_lock_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 10));
			assert_ok!(Tokens::set_lock(ID_2, AXC, &ALICE, 20));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 2);
			assert_ok!(Tokens::remove_lock(ID_2, AXC, &ALICE));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 1);
		});
}

#[test]
fn multi_reservable_currency_can_reserve_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert!(Tokens::can_reserve(AXC, &ALICE, 0));
			assert!(!Tokens::can_reserve(AXC, &ALICE, 101));
			assert!(Tokens::can_reserve(AXC, &ALICE, 100));
		});
}

#[test]
fn multi_reservable_currency_slash_reserved_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::reserve(AXC, &ALICE, 50));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_eq!(Tokens::slash_reserved(AXC, &ALICE, 0), 0);
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_eq!(Tokens::slash_reserved(AXC, &ALICE, 100), 50);
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 50);
		});
}

#[test]
fn multi_reservable_currency_reserve_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_noop!(Tokens::reserve(AXC, &ALICE, 101), Error::<Runtime>::BalanceTooLow);
			assert_ok!(Tokens::reserve(AXC, &ALICE, 0));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::total_balance(AXC, &ALICE), 100);
			assert_ok!(Tokens::reserve(AXC, &ALICE, 50));
			System::assert_last_event(Event::Tokens(crate::Event::Reserved(AXC, ALICE, 50)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::total_balance(AXC, &ALICE), 100);

			assert_ok!(Tokens::reserve(AXC, &ALICE, 50));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::total_balance(AXC, &ALICE), 100);
			// ensure will not trigger Endowed event
			assert!(System::events()
				.iter()
				.all(|record| !matches!(record.event, Event::Tokens(crate::Event::Endowed(AXC, ALICE, _)))));
		});
}

#[test]
fn multi_reservable_currency_unreserve_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::unreserve(AXC, &ALICE, 0), 0);
			assert_eq!(Tokens::unreserve(AXC, &ALICE, 50), 50);
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(AXC, ALICE, 0)));
			assert_ok!(Tokens::reserve(AXC, &ALICE, 30));
			System::assert_last_event(Event::Tokens(crate::Event::Reserved(AXC, ALICE, 30)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 70);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 30);
			assert_eq!(Tokens::unreserve(AXC, &ALICE, 15), 0);
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(AXC, ALICE, 15)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 85);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 15);
			assert_eq!(Tokens::unreserve(AXC, &ALICE, 30), 15);
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(AXC, ALICE, 15)));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			// ensure will not trigger Endowed event
			assert!(System::events()
				.iter()
				.all(|record| !matches!(record.event, Event::Tokens(crate::Event::Endowed(AXC, ALICE, _)))));
		});
}

#[test]
fn multi_reservable_currency_repatriate_reserved_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			assert_eq!(
				Tokens::repatriate_reserved(AXC, &ALICE, &ALICE, 0, BalanceStatus::Free),
				Ok(0)
			);
			assert_eq!(
				Tokens::repatriate_reserved(AXC, &ALICE, &ALICE, 50, BalanceStatus::Free),
				Ok(50)
			);
			// Repatriating from and to the same account, fund is `unreserved`.
			System::assert_last_event(Event::Tokens(crate::Event::Unreserved(AXC, ALICE, 0)));

			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);

			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 0);
			assert_ok!(Tokens::reserve(AXC, &BOB, 50));
			assert_eq!(Tokens::free_balance(AXC, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 50);
			assert_eq!(
				Tokens::repatriate_reserved(AXC, &BOB, &BOB, 60, BalanceStatus::Reserved),
				Ok(10)
			);

			assert_eq!(Tokens::free_balance(AXC, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 50);

			assert_eq!(
				Tokens::repatriate_reserved(AXC, &BOB, &ALICE, 30, BalanceStatus::Reserved),
				Ok(0)
			);
			System::assert_last_event(Event::Tokens(crate::Event::RepatriatedReserve(
				AXC,
				BOB,
				ALICE,
				30,
				BalanceStatus::Reserved,
			)));

			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 30);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 20);

			assert_eq!(
				Tokens::repatriate_reserved(AXC, &BOB, &ALICE, 30, BalanceStatus::Free),
				Ok(10)
			);

			// Actual amount repatriated is 20.
			System::assert_last_event(Event::Tokens(crate::Event::RepatriatedReserve(
				AXC,
				BOB,
				ALICE,
				20,
				BalanceStatus::Free,
			)));

			assert_eq!(Tokens::free_balance(AXC, &ALICE), 120);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 30);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &BOB), 0);
		});
}

#[test]
fn slash_draw_reserved_correct() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::reserve(AXC, &ALICE, 50));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(AXC), 100);

			assert_eq!(Tokens::slash(AXC, &ALICE, 80), 0);
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 20);
			assert_eq!(Tokens::total_issuance(AXC), 20);

			assert_eq!(Tokens::slash(AXC, &ALICE, 50), 30);
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::reserved_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(AXC), 0);
		});
}

#[test]
fn no_op_if_amount_is_zero() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Tokens::ensure_can_withdraw(AXC, &ALICE, 0));
		assert_ok!(Tokens::transfer(Some(ALICE).into(), BOB, AXC, 0));
		assert_ok!(Tokens::transfer(Some(ALICE).into(), ALICE, AXC, 0));
		assert_ok!(Tokens::deposit(AXC, &ALICE, 0));
		assert_ok!(Tokens::withdraw(AXC, &ALICE, 0));
		assert_eq!(Tokens::slash(AXC, &ALICE, 0), 0);
		assert_eq!(Tokens::slash(AXC, &ALICE, 1), 1);
		assert_ok!(Tokens::update_balance(AXC, &ALICE, 0));
	});
}

#[test]
fn transfer_all_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (ALICE, BTC, 200)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 100);
			assert_eq!(Tokens::free_balance(BTC, &ALICE), 200);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 0);

			assert_ok!(<Tokens as TransferAll<AccountId>>::transfer_all(&ALICE, &BOB));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(BTC, &ALICE), 0);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 100);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 200);

			assert_ok!(Tokens::reserve(AXC, &BOB, 1));
			assert_ok!(<Tokens as TransferAll<AccountId>>::transfer_all(&BOB, &ALICE));
			assert_eq!(Tokens::free_balance(AXC, &ALICE), 99);
			assert_eq!(Tokens::free_balance(BTC, &ALICE), 200);
			assert_eq!(Tokens::free_balance(AXC, &BOB), 0);
			assert_eq!(Tokens::free_balance(BTC, &BOB), 0);
		});
}

// *************************************************
// tests for CurrencyAdapter
// *************************************************

#[test]
fn currency_adapter_ensure_currency_adapter_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(Tokens::total_issuance(AXC), 100);
			assert_eq!(Tokens::total_balance(AXC, &TREASURY_ACCOUNT), 100);
			assert_eq!(Tokens::reserved_balance(AXC, &TREASURY_ACCOUNT), 0);
			assert_eq!(Tokens::free_balance(AXC, &TREASURY_ACCOUNT), 100);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_balance(&TREASURY_ACCOUNT),
				100
			);
			assert!(<Runtime as pallet_elections_phragmen::Config>::Currency::can_slash(
				&TREASURY_ACCOUNT,
				10
			));
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				100
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::minimum_balance(),
				2
			);
			assert!(<Runtime as pallet_elections_phragmen::Config>::Currency::can_reserve(
				&TREASURY_ACCOUNT,
				5
			));

			// burn
			let imbalance = <Runtime as pallet_elections_phragmen::Config>::Currency::burn(10);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				90
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				100
			);

			// issue
			let imbalance = <Runtime as pallet_elections_phragmen::Config>::Currency::issue(20);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				120
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				100
			);

			// transfer
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_ok!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::ensure_can_withdraw(
					&TREASURY_ACCOUNT,
					10,
					WithdrawReasons::TRANSFER,
					0
				)
			);
			assert_ok!(<Runtime as pallet_elections_phragmen::Config>::Currency::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				11,
				ExistenceRequirement::KeepAlive
			));
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				89
			);

			// deposit
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				100
			);
			let imbalance = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 11);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				100
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				100
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				111
			);

			// withdraw
			let imbalance = <Runtime as pallet_elections_phragmen::Config>::Currency::withdraw(
				&TREASURY_ACCOUNT,
				10,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::KeepAlive,
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				90
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				111
			);
			drop(imbalance);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::free_balance(&TREASURY_ACCOUNT),
				90
			);
			assert_eq!(
				<Runtime as pallet_elections_phragmen::Config>::Currency::total_issuance(),
				101
			);
		});
}

#[test]
fn currency_adapter_burn_must_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			let init_total_issuance = TreasuryCurrencyAdapter::total_issuance();
			let imbalance = TreasuryCurrencyAdapter::burn(10);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), init_total_issuance - 10);
			drop(imbalance);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), init_total_issuance);
		});
}

#[test]
fn currency_adapter_reserving_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);

		assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);

		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));

		assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 42);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
	});
}

#[test]
fn currency_adapter_balance_transfer_when_reserved_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));
		assert_noop!(
			TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 69, ExistenceRequirement::AllowDeath),
			Error::<Runtime>::BalanceTooLow,
		);
	});
}

#[test]
fn currency_adapter_deducting_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 42);
	});
}

#[test]
fn currency_adapter_refunding_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 42);
		Tokens::set_reserved_balance(AXC, &TREASURY_ACCOUNT, 69);
		TreasuryCurrencyAdapter::unreserve(&TREASURY_ACCOUNT, 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 111);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
	});
}

#[test]
fn currency_adapter_slashing_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 69));
		assert!(TreasuryCurrencyAdapter::slash(&TREASURY_ACCOUNT, 69).1.is_zero());
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 42);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 42);
	});
}

#[test]
fn currency_adapter_slashing_incomplete_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 42);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 42);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 21));
		assert_eq!(TreasuryCurrencyAdapter::slash(&TREASURY_ACCOUNT, 69).1, 27);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 0);
	});
}

#[test]
fn currency_adapter_basic_locking_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 100);
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 91, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 10, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_partial_locking_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 5, WithdrawReasons::all());
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_lock_removal_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, u64::max_value(), WithdrawReasons::all());
			TreasuryCurrencyAdapter::remove_lock(ID_1, &TREASURY_ACCOUNT);
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_lock_replacement_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, u64::max_value(), WithdrawReasons::all());
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 5, WithdrawReasons::all());
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_double_locking_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 5, WithdrawReasons::empty());
			TreasuryCurrencyAdapter::set_lock(ID_2, &TREASURY_ACCOUNT, 5, WithdrawReasons::all());
			assert_ok!(TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				2,
				ExistenceRequirement::AllowDeath
			));
		});
}

#[test]
fn currency_adapter_combination_locking_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			// withdrawReasons not work
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, u64::max_value(), WithdrawReasons::empty());
			TreasuryCurrencyAdapter::set_lock(ID_2, &TREASURY_ACCOUNT, 0, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 2, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_lock_value_extension_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 100, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 2, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 8, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 3, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_lock_block_number_extension_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 200, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 6, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			System::set_block_number(2);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::all());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 3, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_lock_reasons_extension_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			TreasuryCurrencyAdapter::set_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::TRANSFER);
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 11, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::empty());
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 11, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
			TreasuryCurrencyAdapter::extend_lock(ID_1, &TREASURY_ACCOUNT, 90, WithdrawReasons::RESERVE);
			assert_noop!(
				TreasuryCurrencyAdapter::transfer(&TREASURY_ACCOUNT, &ALICE, 11, ExistenceRequirement::AllowDeath),
				Error::<Runtime>::LiquidityRestrictions
			);
		});
}

#[test]
fn currency_adapter_deposit_creating_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 100);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 0);
			let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 2);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 102);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 2);

			assert_eq!(TreasuryCurrencyAdapter::total_balance(&ALICE), 100);
			let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 1);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 103);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&ALICE), 101);
		});
}

#[test]
fn currency_adapter_deposit_into_existing_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 0);
			assert_noop!(
				TreasuryCurrencyAdapter::deposit_into_existing(&TREASURY_ACCOUNT, 10).map(drop),
				Error::<Runtime>::DeadAccount,
			);

			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 100);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&ALICE), 100);
			assert_ok!(TreasuryCurrencyAdapter::deposit_into_existing(&ALICE, 10).map(drop));
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 110);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&ALICE), 110);
		});
}

#[test]
fn currency_adapter_reward_should_work() {
	ExtBuilder::default()
		.balances(vec![(TREASURY_ACCOUNT, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 100);
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 100);
			assert_ok!(TreasuryCurrencyAdapter::deposit_into_existing(&TREASURY_ACCOUNT, 10).map(drop));
			assert_eq!(TreasuryCurrencyAdapter::total_balance(&TREASURY_ACCOUNT), 110);
			assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 110);
		});
}

#[test]
fn currency_adapter_slashing_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 111));
		assert_eq!(TreasuryCurrencyAdapter::slash_reserved(&TREASURY_ACCOUNT, 42).1, 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 69);
	});
}

#[test]
fn currency_adapter_slashing_incomplete_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 42));
		assert_eq!(TreasuryCurrencyAdapter::slash_reserved(&TREASURY_ACCOUNT, 69).1, 27);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::total_issuance(), 69);
	});
}

#[test]
fn currency_adapter_repatriating_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 110);
		let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 2);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 110));
		assert_ok!(
			TreasuryCurrencyAdapter::repatriate_reserved(&TREASURY_ACCOUNT, &ALICE, 41, Status::Free),
			0
		);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&ALICE), 0);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 43);
	});
}

#[test]
fn currency_adapter_transferring_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 110);
		let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 2);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 110));
		assert_ok!(
			TreasuryCurrencyAdapter::repatriate_reserved(&TREASURY_ACCOUNT, &ALICE, 41, Status::Reserved),
			0
		);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&ALICE), 41);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 2);
	});
}

#[test]
fn currency_adapter_transferring_reserved_balance_to_nonexistent_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 111);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 111));
		assert_ok!(TreasuryCurrencyAdapter::repatriate_reserved(
			&TREASURY_ACCOUNT,
			&ALICE,
			42,
			Status::Free
		));
	});
}

#[test]
fn currency_adapter_transferring_incomplete_reserved_balance_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let _ = TreasuryCurrencyAdapter::deposit_creating(&TREASURY_ACCOUNT, 110);
		let _ = TreasuryCurrencyAdapter::deposit_creating(&ALICE, 2);
		assert_ok!(TreasuryCurrencyAdapter::reserve(&TREASURY_ACCOUNT, 41));
		assert_ok!(
			TreasuryCurrencyAdapter::repatriate_reserved(&TREASURY_ACCOUNT, &ALICE, 69, Status::Free),
			28
		);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&TREASURY_ACCOUNT), 0);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT), 69);
		assert_eq!(TreasuryCurrencyAdapter::reserved_balance(&ALICE), 0);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 43);
	});
}

#[test]
fn currency_adapter_transferring_too_high_value_should_not_panic() {
	ExtBuilder::default().build().execute_with(|| {
		TreasuryCurrencyAdapter::make_free_balance_be(&TREASURY_ACCOUNT, u64::max_value());
		TreasuryCurrencyAdapter::make_free_balance_be(&ALICE, 2);

		assert_noop!(
			TreasuryCurrencyAdapter::transfer(
				&TREASURY_ACCOUNT,
				&ALICE,
				u64::max_value(),
				ExistenceRequirement::AllowDeath
			),
			ArithmeticError::Overflow,
		);

		assert_eq!(
			TreasuryCurrencyAdapter::free_balance(&TREASURY_ACCOUNT),
			u64::max_value()
		);
		assert_eq!(TreasuryCurrencyAdapter::free_balance(&ALICE), 2);
	});
}

#[test]
fn exceeding_max_locks_should_fail() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Tokens::set_lock(ID_1, AXC, &ALICE, 10));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 1);
			assert_ok!(Tokens::set_lock(ID_2, AXC, &ALICE, 10));
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 2);
			assert_noop!(
				Tokens::set_lock(ID_3, AXC, &ALICE, 10),
				Error::<Runtime>::MaxLocksExceeded
			);
			assert_eq!(Tokens::locks(ALICE, AXC).len(), 2);
		});
}

// *************************************************
// tests for fungibles traits
// *************************************************

#[test]
fn fungibles_inspect_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::Inspect<_>>::total_issuance(AXC), 100);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::minimum_balance(AXC), 2);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &ALICE), 100);
			assert_eq!(
				<Tokens as fungibles::Inspect<_>>::reducible_balance(AXC, &ALICE, true),
				98
			);
			assert_ok!(<Tokens as fungibles::Inspect<_>>::can_deposit(AXC, &ALICE, 1).into_result());
			assert_ok!(<Tokens as fungibles::Inspect<_>>::can_withdraw(AXC, &ALICE, 1).into_result());
		});
}

#[test]
fn fungibles_mutate_trait_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(<Tokens as fungibles::Mutate<_>>::mint_into(AXC, &ALICE, 10));
		assert_eq!(<Tokens as fungibles::Mutate<_>>::burn_from(AXC, &ALICE, 8), Ok(8));
	});
}

#[test]
fn fungibles_transfer_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &ALICE), 100);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &BOB), 100);
			assert_ok!(<Tokens as fungibles::Transfer<_>>::transfer(
				AXC, &ALICE, &BOB, 10, true
			));
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &ALICE), 90);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &BOB), 110);
		});
}

#[test]
fn fungibles_unbalanced_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &ALICE), 100);
			assert_ok!(<Tokens as fungibles::Unbalanced<_>>::set_balance(AXC, &ALICE, 10));
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &ALICE), 10);

			assert_eq!(<Tokens as fungibles::Inspect<_>>::total_issuance(AXC), 100);
			<Tokens as fungibles::Unbalanced<_>>::set_total_issuance(AXC, 10);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::total_issuance(AXC), 10);
		});
}

#[test]
fn fungibles_inspect_hold_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 0);
			assert!(<Tokens as fungibles::InspectHold<_>>::can_hold(AXC, &ALICE, 50));
			assert!(!<Tokens as fungibles::InspectHold<_>>::can_hold(AXC, &ALICE, 100));
		});
}

#[test]
fn fungibles_mutate_hold_trait_should_work() {
	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				<Tokens as fungibles::MutateHold<_>>::hold(AXC, &ALICE, 200),
				Error::<Runtime>::BalanceTooLow
			);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 0);
			assert_ok!(<Tokens as fungibles::MutateHold<_>>::hold(AXC, &ALICE, 100));
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 100);
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::release(AXC, &ALICE, 40, false),
				Ok(40)
			);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 60);

			// exceed hold amount when not in best_effort
			assert_noop!(
				<Tokens as fungibles::MutateHold<_>>::release(AXC, &ALICE, 61, false),
				Error::<Runtime>::BalanceTooLow
			);

			// exceed hold amount when in best_effort
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::release(AXC, &ALICE, 61, true),
				Ok(60)
			);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 0);

			assert_ok!(<Tokens as fungibles::MutateHold<_>>::hold(AXC, &ALICE, 70));
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 70);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &BOB), 100);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &BOB), 0);
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::transfer_held(AXC, &ALICE, &BOB, 5, false, false),
				Ok(5)
			);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 65);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &BOB), 105);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &BOB), 0);
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::transfer_held(AXC, &ALICE, &BOB, 5, false, true),
				Ok(5)
			);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 60);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &BOB), 110);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &BOB), 5);

			// exceed hold amount when not in best_effort
			assert_noop!(
				<Tokens as fungibles::MutateHold<_>>::transfer_held(AXC, &ALICE, &BOB, 61, false, true),
				Error::<Runtime>::BalanceTooLow
			);

			// exceed hold amount when in best_effort
			assert_eq!(
				<Tokens as fungibles::MutateHold<_>>::transfer_held(AXC, &ALICE, &BOB, 61, true, true),
				Ok(60)
			);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &ALICE), 0);
			assert_eq!(<Tokens as fungibles::Inspect<_>>::balance(AXC, &BOB), 170);
			assert_eq!(<Tokens as fungibles::InspectHold<_>>::balance_on_hold(AXC, &BOB), 65);
		});
}

#[test]
fn fungibles_inspect_convert_should_work() {
	pub struct ConvertBalanceTest;
	impl ConvertBalance<Balance, Balance> for ConvertBalanceTest {
		type AssetId = CurrencyId;
		fn convert_balance(balance: Balance, _asset_id: CurrencyId) -> Balance {
			balance * 100
		}

		fn convert_balance_back(balance: Balance, _asset_id: CurrencyId) -> Balance {
			balance / 100
		}
	}

	pub struct IsLiquidToken;
	impl Contains<CurrencyId> for IsLiquidToken {
		fn contains(currency_id: &CurrencyId) -> bool {
			matches!(currency_id, &AXC)
		}
	}

	pub struct GetCurrencyId;
	impl Get<CurrencyId> for GetCurrencyId {
		fn get() -> CurrencyId {
			AXC
		}
	}

	type RebaseTokens = Combiner<
		AccountId,
		IsLiquidToken,
		Mapper<AccountId, Tokens, ConvertBalanceTest, Balance, GetCurrencyId>,
		Tokens,
	>;

	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 100), (BOB, AXC, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(
				<RebaseTokens as fungibles::Inspect<AccountId>>::balance(AXC, &ALICE),
				10000
			);
			assert_eq!(
				<RebaseTokens as fungibles::Inspect<AccountId>>::total_issuance(AXC),
				20000
			);
		});
}

#[test]
fn fungibles_transfers_convert_should_work() {
	pub struct ConvertBalanceTest;
	impl ConvertBalance<Balance, Balance> for ConvertBalanceTest {
		type AssetId = CurrencyId;
		fn convert_balance(balance: Balance, _asset_id: CurrencyId) -> Balance {
			balance * 100
		}

		fn convert_balance_back(balance: Balance, _asset_id: CurrencyId) -> Balance {
			balance / 100
		}
	}

	pub struct IsLiquidToken;
	impl Contains<CurrencyId> for IsLiquidToken {
		fn contains(currency_id: &CurrencyId) -> bool {
			matches!(currency_id, &AXC)
		}
	}

	pub struct GetCurrencyId;
	impl Get<CurrencyId> for GetCurrencyId {
		fn get() -> CurrencyId {
			AXC
		}
	}

	type RebaseTokens = Combiner<
		AccountId,
		IsLiquidToken,
		Mapper<AccountId, Tokens, ConvertBalanceTest, Balance, GetCurrencyId>,
		Tokens,
	>;

	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 300), (BOB, AXC, 200)])
		.build()
		.execute_with(|| {
			assert_ok!(<RebaseTokens as fungibles::Transfer<AccountId>>::transfer(
				AXC, &ALICE, &BOB, 10000, true
			));
			assert_eq!(
				<RebaseTokens as fungibles::Inspect<AccountId>>::balance(AXC, &ALICE),
				20000
			);
			assert_eq!(
				<RebaseTokens as fungibles::Inspect<AccountId>>::balance(AXC, &BOB),
				30000
			);
		});
}

#[test]
fn fungibles_mutate_convert_should_work() {
	pub struct ConvertBalanceTest;
	impl ConvertBalance<Balance, Balance> for ConvertBalanceTest {
		type AssetId = CurrencyId;
		fn convert_balance(balance: Balance, _asset_id: CurrencyId) -> Balance {
			balance * 100
		}

		fn convert_balance_back(balance: Balance, _asset_id: CurrencyId) -> Balance {
			balance / 100
		}
	}

	pub struct IsLiquidToken;
	impl Contains<CurrencyId> for IsLiquidToken {
		fn contains(currency_id: &CurrencyId) -> bool {
			matches!(currency_id, &AXC)
		}
	}

	pub struct GetCurrencyId;
	impl Get<CurrencyId> for GetCurrencyId {
		fn get() -> CurrencyId {
			AXC
		}
	}

	type RebaseTokens = Combiner<
		AccountId,
		IsLiquidToken,
		Mapper<AccountId, Tokens, ConvertBalanceTest, Balance, GetCurrencyId>,
		Tokens,
	>;

	ExtBuilder::default()
		.balances(vec![(ALICE, AXC, 300), (BOB, AXC, 200)])
		.build()
		.execute_with(|| {
			assert_ok!(<RebaseTokens as fungibles::Mutate<AccountId>>::mint_into(
				AXC, &ALICE, 10000
			));
			assert_ok!(<RebaseTokens as fungibles::Mutate<AccountId>>::burn_from(
				AXC, &BOB, 10000
			));
			assert_eq!(
				<RebaseTokens as fungibles::Inspect<AccountId>>::balance(AXC, &ALICE),
				40000
			);
			assert_eq!(
				<RebaseTokens as fungibles::Inspect<AccountId>>::balance(AXC, &BOB),
				10000
			);
		});
}
