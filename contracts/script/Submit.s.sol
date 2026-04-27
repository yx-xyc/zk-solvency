// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {SolvencyAttestation} from "../src/SolvencyAttestation.sol";

/// Forge script that submits a generated proof on-chain.
///
/// Required env vars:
///   CONTRACT_ADDRESS  — deployed SolvencyAttestation address
///   PRIVATE_KEY       — hex private key for the broadcaster account
///   PROOF_BYTES       — hex proof bytes (from proof.json .proof_bytes)
///   PUBLIC_VALUES     — hex public values (from proof.json .public_values)
///
/// Run (extract values with jq, then broadcast):
///   CONTRACT_ADDRESS=0x... PRIVATE_KEY=0x... \
///   PROOF_BYTES=$(jq -r '.proof_bytes' proof.json) \
///   PUBLIC_VALUES=$(jq -r '.public_values' proof.json) \
///   forge script contracts/script/Submit.s.sol \
///     --rpc-url <RPC_URL> --broadcast
contract Submit is Script {
    function run() external {
        bytes memory proofBytes   = vm.envBytes("PROOF_BYTES");
        bytes memory publicValues = vm.envBytes("PUBLIC_VALUES");
        address contractAddress   = vm.envAddress("CONTRACT_ADDRESS");
        uint256 privateKey        = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(privateKey);
        SolvencyAttestation(contractAddress).submitProof(proofBytes, publicValues);
        vm.stopBroadcast();

        console.log("Proof submitted to", contractAddress);
    }
}
