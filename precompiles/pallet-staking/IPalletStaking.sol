// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

// https://docs.substrate.io/rustdocs/latest/pallet_staking/pallet/enum.Call.html#variant.chill

// precompile address: 0x0000000000000000000000000000000000000808 (=2056)
interface IPalletStaking {
    function chill() external;
    function bond(uint256 _amount) external;
    function unbond(uint256 _amount) external;
    function nominate(bytes32[] _targets) external;
}

contract PalletStaking {
    function chill() external {
        IPalletStaking(0x0000000000000000000000000000000000000808).chill();
    }
    function bond(uint256 _amount) external {
        IPalletStaking(0x0000000000000000000000000000000000000808).bond(_amount);
    }
    function unbond(uint256 _amount) external {
        IPalletStaking(0x0000000000000000000000000000000000000808).unbond(_amount);
    }
    function nominate(bytes32[] _targets) external {
        IPalletStaking(0x0000000000000000000000000000000000000808).nominate(_targets);
    }
}
