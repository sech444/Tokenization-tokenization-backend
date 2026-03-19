// contracts/interfaces/core/IAdminGovernance.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IAdminGovernance
 * @dev The complete interface for the AdminGovernance contract.
 */
interface IAdminGovernance {
    // --- Enums ---
    enum ProposalState { PENDING, ACTIVE, SUCCEEDED, DEFEATED, EXECUTED, CANCELLED, EXPIRED }
    enum VoteType { AGAINST, FOR, ABSTAIN }
    
    // ===== THIS IS THE FIX: Add the missing initialize function =====
    function initialize(address admin, address auditTrail) external;

    // --- Other Functions ---
    function propose(
        string calldata title,
        string calldata description,
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas
    ) external returns (uint256);
    
    function castVote(uint256 proposalId, VoteType support) external;
    function execute(uint256 proposalId) external payable;
    function cancel(uint256 proposalId) external;
    
    function state(uint256 proposalId) external view returns (ProposalState);
    function getProposal(uint256 proposalId) external view returns (
        uint256 id,
        address proposer,
        string memory title,
        string memory description,
        uint256 startBlock,
        uint256 endBlock,
        uint256 forVotes,
        uint256 againstVotes,
        uint256 abstainVotes,
        ProposalState currentState,
        bool executed,
        uint256 eta
    );
}