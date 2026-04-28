// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {SolvencyAttestation} from "../src/SolvencyAttestation.sol";

/// Forge script that deploys SolvencyAttestation to Sepolia.
///
/// Required env vars:
///   PRIVATE_KEY  — hex private key for the deployer account
///
/// Run:
///   PRIVATE_KEY=0x... \
///   forge script contracts/script/Deploy.s.sol \
///     --rpc-url <SEPOLIA_RPC_URL> --broadcast
///
/// Copy the printed contract address into deployment.json and .env as CONTRACT_ADDRESS.
contract Deploy is Script {
    address constant SP1_VERIFIER =
        0xd685a80aF2d1761648e56716af4868d850Dae49B;
    bytes32 constant PROGRAM_VKEY =
        0x0098ee1f091411258d9318cb9a146c4e48145cee16b45a774d0445772cbfca4f;

    function run() external returns (address) {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(privateKey);
        SolvencyAttestation attestation = new SolvencyAttestation(SP1_VERIFIER, PROGRAM_VKEY);
        vm.stopBroadcast();
        console.log("SolvencyAttestation deployed to:", address(attestation));
        return address(attestation);
    }
}
