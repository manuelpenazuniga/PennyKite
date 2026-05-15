// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console2} from "forge-std/Test.sol";
import {PennyKiteAttestor} from "../src/PennyKiteAttestor.sol";

contract PennyKiteAttestorTest is Test {
    PennyKiteAttestor public attestor;

    function setUp() public {
        attestor = new PennyKiteAttestor();
    }

    function test_SingleAttest() public {
        bytes32 sessionId = bytes32(uint256(1));
        bytes32 decisionHash = keccak256("test decision");

        vm.expectEmit(true, true, true, true);
        emit PennyKiteAttestor.Attested(sessionId, decisionHash, block.timestamp, address(this), 0);

        uint256 index = attestor.attest(sessionId, decisionHash);
        assertEq(index, 0);
        assertEq(attestor.totalAttestations(), 1);
    }

    function test_MultipleAttestationsPerSession() public {
        bytes32 sessionId = bytes32(uint256(1));
        attestor.attest(sessionId, keccak256("d1"));
        attestor.attest(sessionId, keccak256("d2"));
        attestor.attest(sessionId, keccak256("d3"));

        uint256[] memory indices = attestor.getSessionAttestations(sessionId);
        assertEq(indices.length, 3);
        assertEq(indices[0], 0);
        assertEq(indices[1], 1);
        assertEq(indices[2], 2);
    }

    function test_TotalAttestationsMonotonic() public {
        assertEq(attestor.totalAttestations(), 0);
        attestor.attest(bytes32(uint256(1)), keccak256("a"));
        assertEq(attestor.totalAttestations(), 1);
        attestor.attest(bytes32(uint256(2)), keccak256("b"));
        assertEq(attestor.totalAttestations(), 2);
    }

    function test_NonOwnerCanAttest() public {
        address alice = makeAddr("alice");
        vm.prank(alice);
        uint256 index = attestor.attest(bytes32(uint256(42)), keccak256("alice decision"));
        assertEq(index, 0);

        PennyKiteAttestor.Attestation memory a = attestor.getAttestation(0);
        assertEq(a.attester, alice);
    }

    function test_RevertOnOutOfBounds() public {
        vm.expectRevert("index out of bounds");
        attestor.getAttestation(0);
    }
}
