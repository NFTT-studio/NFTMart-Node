// SPDX-License-Identifier: GPL-3.0-only

pragma solidity >=0.8.0;

interface INftmartOrder {
    function removeOffer(uint _offerId) external;
    function removeOrder(uint _orderId) external;
    function submitOffer(uint _currencyId, uint256 _price, uint _deadline, uint[3][] memory _items, uint _commissionRate) external;
    function submitOrder(uint _currencyId, uint256 _deposit, uint256 _price, uint _deadline, uint[3][] memory _items, uint _commissionRate) external;
    function takeOffer(uint _offerId, bytes32 _offerOwner, bytes32 _commissionAgent, string memory _commissionData) external;
    function takeOrder(uint _orderId, bytes32 _orderOwner, bytes32 _commissionAgent, string memory _commissionData) external;
}
