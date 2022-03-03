// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

// https://docs.substrate.io/rustdocs/latest/pallet_staking/pallet/enum.Call.html#variant.chill

// precompile address: 0x0000000000000000000000000000000000000808 (=2056)
interface IPalletStaking {
    function chill() external;
}

contract PalletStaking {
    function chill() external {
        IPalletStaking(0x0000000000000000000000000000000000000808).chill();
    }
}
