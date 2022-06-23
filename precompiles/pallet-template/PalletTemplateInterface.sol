// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with Crowdloan Rewards
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
interface PalletTemplate {

    /// @dev Store a value in the pallet's storage.
    /// @param value The new value to store. 32 bit maximum
    function do_something(uint256 value) external;

    /// @dev Retrieve the stored value
    /// @return A uint256 (with max 32-bit value) indicating the value stored in the pallet
    function get_value() external view returns (uint256);
}
