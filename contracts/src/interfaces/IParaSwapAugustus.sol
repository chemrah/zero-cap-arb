// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title IParaSwapAugustus
/// @notice ParaSwap V5 Augustus Swapper interface
/// @dev The Augustus contract receives calldata built by the ParaSwap API
///      and executes the multi-step swaps internally.
interface IParaSwapAugustus {
    /// @notice Perform a simple swap through ParaSwap
    /// @param data The calldata from ParaSwap API buildTransaction
    /// @return receivedAmount The amount of dest tokens received
    function simpleSwap(bytes calldata data) external payable returns (uint256 receivedAmount);

    /// @notice Perform a mega swap (multi-step, split-route) through ParaSwap
    /// @param data The calldata from ParaSwap API buildTransaction for megaSwap
    /// @return receivedAmount The amount of dest tokens received
    function megaSwap(bytes calldata data) external payable returns (uint256 receivedAmount);

    /// @notice Get the token transfer proxy address
    function getTokenTransferProxy() external view returns (address);
}
