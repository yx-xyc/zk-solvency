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
        0x397A5f7f3dBd538f23DE225B51f532c34448dA9B;
    bytes32 constant PROGRAM_VKEY =
        0x00680f24d7f1c5c844c2852e84244b6a34215092dc492599792cee4304fd15dd;

    function run() external returns (address) {
        uint256 privateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(privateKey);
        SolvencyAttestation attestation = new SolvencyAttestation(SP1_VERIFIER, PROGRAM_VKEY);
        vm.stopBroadcast();
        console.log("SolvencyAttestation deployed to:", address(attestation));
        return address(attestation);
    }
}
