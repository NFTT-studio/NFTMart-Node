// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

// https://docs.substrate.io/rustdocs/latest/pallet_identity/pallet/enum.Call.html#variant.set_identity

// precompile address: 0x0000000000000000000000000000000000000807 (=2055)
interface IPalletIdentity {
    function chill() external;
}

contract PalletIdentity {
    function chill() external {
        IPalletIdentity(0x0000000000000000000000000000000000000807).chill();
    }
}
