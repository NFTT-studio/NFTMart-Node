// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with Crowdloan Rewards
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
interface WithdrawBalance {
    function withdraw_balance(bytes32, uint256) external;
}
