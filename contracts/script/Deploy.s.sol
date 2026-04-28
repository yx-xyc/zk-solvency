// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {SolvencyAttestation} from "../src/SolvencyAttestation.sol";

/// Forge script that deploys SolvencyAttestation to Sepolia.
///
/// Required env vars:
///   PRIVATE_KEY    — hex private key for the deployer account
///   PROGRAM_VKEY   — bytes32 program vkey from proof.json (jq -r '.program_vkey' proof.json)
///
/// Run (from repo root):
///   PRIVATE_KEY=0x... \
///   PROGRAM_VKEY=$(jq -r '.program_vkey' proof.json) \
///   forge script contracts/script/Deploy.s.sol:Deploy \
///     --root contracts --rpc-url <SEPOLIA_RPC_URL> --broadcast
///
/// Copy the printed contract address into deployment.json.
contract Deploy is Script {
    address constant SP1_VERIFIER =
        0xd685a80aF2d1761648e56716af4868d850Dae49B;

    function run() external returns (address) {
        uint256 privateKey  = vm.envUint("PRIVATE_KEY");
        bytes32 programVKey = vm.envBytes32("PROGRAM_VKEY");

        vm.startBroadcast(privateKey);
        SolvencyAttestation attestation = new SolvencyAttestation(SP1_VERIFIER, programVKey);
        vm.stopBroadcast();

        console.log("SolvencyAttestation deployed to:", address(attestation));
        console.logBytes32(programVKey);
        return address(attestation);
    }
}
