// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title PennyKiteAttestor
/// @notice Append-only on-chain registry for PennyKite budget decisions.
///
/// Every decision (approve or deny) is hashed and anchored here so the
/// audit trail is cryptographically verifiable and independent of the
/// off-chain proxy host.
contract PennyKiteAttestor {
    struct Attestation {
        bytes32 sessionId;
        bytes32 decisionHash;
        uint256 timestamp;
        address attester;
    }

    /// @notice Attestation records indexed by session id.
    mapping(bytes32 => uint256[]) private attestationsBySession;

    /// @notice All attestations ever recorded.
    Attestation[] private _attestations;

    /// @notice Emitted when a new decision is attested.
    event Attested(
        bytes32 indexed sessionId,
        bytes32 indexed decisionHash,
        uint256 timestamp,
        address attester,
        uint256 index
    );

    /// @notice Record a decision on-chain.
    /// @param sessionId The session identifier.
    /// @param decisionHash SHA3-256 hash of the canonical JSON decision.
    /// @return index The global index of the new attestation.
    function attest(bytes32 sessionId, bytes32 decisionHash)
        external
        returns (uint256 index)
    {
        index = _attestations.length;
        _attestations.push(Attestation({
            sessionId: sessionId,
            decisionHash: decisionHash,
            timestamp: block.timestamp,
            attester: msg.sender
        }));
        attestationsBySession[sessionId].push(index);
        emit Attested(sessionId, decisionHash, block.timestamp, msg.sender, index);
    }

    /// @notice Get all attestation indices for a session.
    function getSessionAttestations(bytes32 sessionId)
        external
        view
        returns (uint256[] memory)
    {
        return attestationsBySession[sessionId];
    }

    /// @notice Total number of attestations recorded.
    function totalAttestations() external view returns (uint256) {
        return _attestations.length;
    }

    /// @notice Get a single attestation by index.
    function getAttestation(uint256 index)
        external
        view
        returns (Attestation memory)
    {
        require(index < _attestations.length, "index out of bounds");
        return _attestations[index];
    }
}
