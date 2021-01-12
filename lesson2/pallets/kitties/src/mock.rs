use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::traits::*;

impl_outer_origin! {
	pub enum Origin for Test {}
}

mod kitties {
	pub use crate::Event;
}
impl_outer_event! {
	pub enum Event for Test {
		system<T>,
		kitties<T>,
	}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const KittyIndexUnit: u32 = 1;
	pub const KittyIndexMaxValue: u32 = u32::MAX;
	pub const ExistentialDeposit: u64 = 10;
}

impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

type Randomness = pallet_randomness_collective_flip::Module<Test>;

impl Trait for Test {
	type Event = Event;
	type Randomness = Randomness;
	type KittyIndex = u32;
    type KittyIndexUnit = KittyIndexUnit;
	type KittyIndexMaxValue = KittyIndexMaxValue;
	type Balance = u64;
	type ExistentialDeposit = ExistentialDeposit;
}

pub type KittiesModule = Module<Test>;
pub type System = frame_system::Module<Test>;

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		KittiesModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number()+1);
		System::on_initialize(System::block_number());
		KittiesModule::on_initialize(System::block_number());
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn events() -> Vec<<Test as Trait>::Event> {
	let evt = System::events().into_iter().map(|evt| evt.event).collect::<Vec<_>>();
	System::reset_events();
	evt
}

pub fn last_event() -> <Test as Trait>::Event {
	system::Module::<Test>::events().pop().expect("Event expected").event
}
