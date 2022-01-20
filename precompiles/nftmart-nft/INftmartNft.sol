// SPDX-License-Identifier: GPL-3.0-only

pragma solidity >=0.8.0;

interface INftmartNft {
    function burn(uint _classId, uint tokenId, uint _quantity) external;
    function createClass(string memory _metadata, string memory _name, string memory _description, uint _royaltyRate, uint8 _properties, uint[] memory _categoryIds) external;
    function destroyClass(uint _classId, bytes32 _dest) external;
    function mint(bytes32 _to, uint _classId, string memory _metadata, uint _quantity, uint _chargeRoyalty) external;
    function proxyMint(bytes32 _to, uint _classId, string memory _metadata, uint _quantity, uint _chargeRoyalty) external;
    function transfer(bytes32 _to, uint[3] memory _items) external;
    function updateClass(uint _classId, string memory _metadata, string memory _name, string memory _description, uint _royaltyRate, uint8 _properties, uint[] memory _categoryIds) external;
    function updateToken(bytes32 _to, uint _classId, uint _tokenId, uint _quantity, string memory _metadata, uint _chargeRoyalty) external;
    function updateTokenMetadata(uint _classId, uint _tokenId, string memory _metadata) external;
    function updateTokenRoyalty(uint _classId, uint _tokenId, uint _chargeRoyalty) external;
    function updateTokenRoyaltyBeneficiary(uint _classId, uint _tokenId, bytes32 _to) external;
}
