use crate::{Module, Trait};
use balances;
use frame_support::{
    impl_outer_event, impl_outer_origin, parameter_types,
    traits::{OnFinalize, OnInitialize},
    weights::Weight,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

mod kitties {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        balances<T>,
        kitties<T>,
    }
}

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const ExistentialDeposit: u64 = 1;
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
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type PalletInfo = ();
}

type Randomness = pallet_randomness_collective_flip::Module<Test>;

parameter_types! {
    pub const Reserve: u64 = 1_000;
}

impl Trait for Test {
    type Event = TestEvent;
    type Randomness = Randomness;
    type KittyIndex = u32;
    type Reserve = Reserve;
    type Currency = balances::Module<Self>;
}

impl balances::Trait for Test {
    type Balance = u64;
    type MaxLocks = ();
    type Event = TestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<Test>;
    type WeightInfo = ();
}

pub type KittiesModule = Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Balances = balances::Module<Test>;

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        KittiesModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        KittiesModule::on_initialize(System::block_number());
    }
}

pub fn last_event() -> TestEvent {
    system::Module::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
