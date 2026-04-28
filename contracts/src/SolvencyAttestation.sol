// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "sp1-contracts/src/ISP1Verifier.sol";

contract SolvencyAttestation {
    ISP1Verifier public immutable VERIFIER;
    bytes32      public immutable PROGRAM_VKEY;

    struct Attestation {
        bytes32 merkleRoot;
        bytes32 assetsCommitment;
        uint64  totalLiabilities;
        uint64  totalAssets;
        uint256 timestamp;
    }

    Attestation public latestAttestation;

    event SolvencyProven(
        bytes32 indexed merkleRoot,
        bytes32 assetsCommitment,
        uint64  totalLiabilities,
        uint64  totalAssets,
        uint256 timestamp
    );

    constructor(address _verifier, bytes32 _programVKey) {
        VERIFIER     = ISP1Verifier(_verifier);
        PROGRAM_VKEY = _programVKey;
    }

    function submitProof(
        bytes calldata proofBytes,
        bytes calldata publicValues
    ) external {
        VERIFIER.verifyProof(PROGRAM_VKEY, publicValues, proofBytes);

        (bytes32 merkleRoot, bytes32 assetsCommitment, uint64 totalLiabilities, uint64 totalAssets) =
            abi.decode(publicValues, (bytes32, bytes32, uint64, uint64));

        latestAttestation = Attestation({
            merkleRoot:       merkleRoot,
            assetsCommitment: assetsCommitment,
            totalLiabilities: totalLiabilities,
            totalAssets:      totalAssets,
            timestamp:        block.timestamp
        });

        emit SolvencyProven(merkleRoot, assetsCommitment, totalLiabilities, totalAssets, block.timestamp);
    }
}
