// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

// precompile address: 0x0000000000000000000000000000000000000809 (=2057)
interface IFrameSystem {
    function remarkWithEvent(bytes memory remark) external;
}

contract FrameSystem {
    function ayyyy(bytes memory remark) external {
        IFrameSystem(0x0000000000000000000000000000000000000809).remarkWithEvent(remark);
    }
}
