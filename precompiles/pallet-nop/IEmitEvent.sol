// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

// precompile address: 0x0000000000000000000000000000000000000806
interface IEmitEvent {
    function emitEvent() external;
}

contract EmitEvent {
    function ayyyy() public {
        IEmitEvent(0x0000000000000000000000000000000000000806).emitEvent();
    }
}
