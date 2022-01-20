// SPDX-License-Identifier: GPL-3.0-only

pragma solidity >=0.8.0;

interface INftmartAuction {
    // function redeemBritishAuction(bytes32 _auctionOwner, uint _auctionId) external;
    // function redeemDutchAuction(bytes32 _auctionOwner, uint _auctionId) external;
    // function removeExpiredBritishAuction(uint _classId, uint _tokenId, bytes32 _to) external;
    // function removeExpiredDutchAuction(uint _classId, uint _tokenId, bytes32 _to) external;
    function bidBritishAuction(uint256 _price, bytes32 _auctionOwner, uint _auctionId, bytes32 _commissionAgent, string memory _commissionData) external;
    function bidDutchAuction(uint256 _price, bytes32 _auctionOwner, uint _auctionId, bytes32 _commissionAgent, string memory _commissionData) external;
    function removeBritishAuction(uint _auctionId) external;
    function removeDutchAuction(uint _auctionId) external;
    function submitBritishAuction(uint _currencyId, uint256 _hammerPrice, uint _minRaise, uint256 _deposit, uint256 _initPrice, uint _deadline, bool _allowDelay, uint[3][] memory _items, uint _commissionRate) external;
    function submitDutchAuction(uint _currencyId, uint256 _hammerPrice, uint _minRaise, uint256 _deposit, uint256 _initPrice, uint _deadline, bool _allowDelay, uint[3][] memory _items, uint _commissionRate) external;
}
