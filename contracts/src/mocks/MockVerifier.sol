// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IVerifier} from "../interfaces/IVerifier.sol";

/// @title MockVerifier
/// @notice Mock verifier for testing purposes
/// @dev Always returns true - DO NOT USE IN PRODUCTION
contract MockVerifier is IVerifier {
    bool public shouldVerify = true;

    function setVerifyResult(bool _shouldVerify) external {
        shouldVerify = _shouldVerify;
    }

    function verify(
        bytes calldata /* _proof */,
        bytes32[] calldata /* _publicInputs */
    ) external view override returns (bool) {
        return shouldVerify;
    }
}
