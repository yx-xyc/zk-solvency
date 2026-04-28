// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {SolvencyAttestation} from "../src/SolvencyAttestation.sol";
import {ISP1Verifier} from "sp1-contracts/src/ISP1Verifier.sol";

/// Mock verifier that always accepts any proof.
contract MockSP1Verifier is ISP1Verifier {
    function verifyProof(bytes32, bytes calldata, bytes calldata) external pure override {}
}

/// Mock verifier that always rejects — simulates a bad proof or wrong vkey.
contract RejectingMockVerifier is ISP1Verifier {
    function verifyProof(bytes32, bytes calldata, bytes calldata) external pure override {
        revert("proof rejected");
    }
}

contract SolvencyAttestationTest is Test {
    event SolvencyProven(
        bytes32 indexed merkleRoot,
        bytes32 assetsCommitment,
        uint64  totalLiabilities,
        uint64  totalAssets,
        uint256 timestamp
    );

    SolvencyAttestation attestation;
    MockSP1Verifier     mockVerifier;

    bytes32 constant VKEY              = bytes32(uint256(0xdeadbeef));
    bytes32 constant MERKLE_ROOT       = bytes32(uint256(0xabcdef));
    bytes32 constant ASSETS_COMMITMENT = bytes32(uint256(0x1234abc));
    uint64  constant LIABILITIES       = 500_000;
    uint64  constant ASSETS            = 600_000;

    function setUp() public {
        mockVerifier = new MockSP1Verifier();
        attestation  = new SolvencyAttestation(address(mockVerifier), VKEY);
    }

    // ─── Happy-path tests ────────────────────────────────────────────────────

    function test_submitProof_storesAttestation() public {
        bytes memory publicValues = abi.encode(MERKLE_ROOT, ASSETS_COMMITMENT, LIABILITIES, ASSETS);
        bytes memory proofBytes   = hex"deadbeef";

        attestation.submitProof(proofBytes, publicValues);

        (bytes32 root, bytes32 commitment, uint64 liabilities, uint64 assets, uint256 ts) =
            attestation.latestAttestation();

        assertEq(root,        MERKLE_ROOT);
        assertEq(commitment,  ASSETS_COMMITMENT);
        assertEq(liabilities, LIABILITIES);
        assertEq(assets,      ASSETS);
        assertGt(ts,          0);
    }

    function test_submitProof_emitsEvent() public {
        bytes memory publicValues = abi.encode(MERKLE_ROOT, ASSETS_COMMITMENT, LIABILITIES, ASSETS);
        bytes memory proofBytes   = hex"deadbeef";

        vm.expectEmit(true, false, false, true);
        emit SolvencyProven(MERKLE_ROOT, ASSETS_COMMITMENT, LIABILITIES, ASSETS, block.timestamp);

        attestation.submitProof(proofBytes, publicValues);
    }

    // Validates ABI encoding produced by the Rust script matches Solidity's abi.decode.
    // Captured from: SP1_PROVER=mock cargo run --manifest-path script/Cargo.toml --bin script
    // Layout: merkleRoot(32) | assetsCommitment(32) | totalLiabilities(32) | totalAssets(32)
    function test_realPublicValues_decodedCorrectly() public {
        bytes memory publicValues = hex"c62b97ef52f4d1c1139f3d829235bfa7510b43beb1da0bf0d1b2f961452bb41be9f2b15ad68b93aeb0dca75ab6fb017446e547ad0b2725d78405d5dbefb2f23a000000000000000000000000000000000000000000000000000000000007a60a0000000000000000000000000000000000000000000000000000000000092da5";

        attestation.submitProof(hex"", publicValues);

        (bytes32 root,, uint64 liabilities, uint64 assets,) = attestation.latestAttestation();

        assertEq(root,        bytes32(0xc62b97ef52f4d1c1139f3d829235bfa7510b43beb1da0bf0d1b2f961452bb41b));
        assertEq(liabilities, 501_258);
        assertEq(assets,      601_509);
        assertGe(assets,      liabilities);
    }

    function test_submitProof_overwritesPreviousAttestation() public {
        bytes memory pv1 = abi.encode(MERKLE_ROOT, ASSETS_COMMITMENT, LIABILITIES, ASSETS);
        attestation.submitProof(hex"", pv1);

        bytes32 newRoot = bytes32(uint256(0x111111));
        bytes memory pv2 = abi.encode(newRoot, bytes32(uint256(0xdeadbeef)), uint64(100), uint64(200));
        attestation.submitProof(hex"", pv2);

        (bytes32 root,,,,) = attestation.latestAttestation();
        assertEq(root, newRoot);
    }

    // ─── Negative / rejection tests ──────────────────────────────────────────

    /// A bad or tampered proof must not update state.
    function test_submitProof_revertsOnVerifierRejection() public {
        RejectingMockVerifier rejectingVerifier = new RejectingMockVerifier();
        SolvencyAttestation   strict = new SolvencyAttestation(address(rejectingVerifier), VKEY);

        bytes memory publicValues = abi.encode(MERKLE_ROOT, ASSETS_COMMITMENT, LIABILITIES, ASSETS);
        vm.expectRevert();
        strict.submitProof(hex"deadbeef", publicValues);
    }

    /// Truncated public values must cause abi.decode to revert.
    function test_submitProof_revertsOnMalformedPublicValues() public {
        bytes memory shortValues = abi.encode(MERKLE_ROOT); // 32 bytes; needs 128
        vm.expectRevert();
        attestation.submitProof(hex"", shortValues);
    }

    /// Before any proof is submitted the attestation fields are zero.
    function test_latestAttestation_isZeroInitially() public view {
        (bytes32 root, bytes32 commitment, uint64 liabilities, uint64 assets, uint256 ts) =
            attestation.latestAttestation();
        assertEq(root,        bytes32(0));
        assertEq(commitment,  bytes32(0));
        assertEq(liabilities, 0);
        assertEq(assets,      0);
        assertEq(ts,          0);
    }
}
