// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

// precompile address: 0x0000000000000000000000000000000000000808 (=2056)
interface IPalletStaking {
    function remarkWithEvent(bytes memory remark) external;
}

contract PalletStaking {
    function ayyyy(bytes memory remark) external {
        IFrameSystem(0x0000000000000000000000000000000000000808).remarkWithEvent(remark);
    }
}
