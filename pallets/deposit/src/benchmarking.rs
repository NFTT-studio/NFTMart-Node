//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Deposit;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

impl_benchmark_test_suite!(Deposit, crate::mock::new_test_ext(), crate::mock::Test,);
