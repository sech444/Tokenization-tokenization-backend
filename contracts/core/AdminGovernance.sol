// contracts/core/AdminGovernance.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/utils/PausableUpgradeable.sol";
import "../interfaces/core/IAuditTrail.sol";

/**
 * @title AdminGovernance
 * @dev Multi-signature governance system for platform administration
 * @notice Implements proposal-based governance with voting and execution
 */
contract AdminGovernance is
    Initializable,
    AccessControlUpgradeable,
    PausableUpgradeable
{
    bytes32 public constant GOVERNOR_ROLE = keccak256("GOVERNOR_ROLE");
    bytes32 public constant PROPOSER_ROLE = keccak256("PROPOSER_ROLE");
    bytes32 public constant EXECUTOR_ROLE = keccak256("EXECUTOR_ROLE");

    enum ProposalState {
        PENDING,
        ACTIVE,
        SUCCEEDED,
        DEFEATED,
        EXECUTED,
        CANCELLED,
        EXPIRED
    }
    enum VoteType {
        AGAINST,
        FOR,
        ABSTAIN
    }

    struct Proposal {
        uint256 id;
        address proposer;
        string title;
        string description;
        address[] targets;
        uint256[] values;
        bytes[] calldatas;
        uint256 startBlock;
        uint256 endBlock;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        ProposalState state;
        bool executed;
        uint256 eta;
        mapping(address => bool) hasVoted;
        mapping(address => VoteType) votes;
        mapping(address => uint256) votingPower;
    }

    struct Governor {
        address governor;
        uint256 votingPower;
        bool isActive;
        uint256 joinedAt;
        string name;
        string contact;
    }

    struct ExecutionQueue {
        uint256 proposalId;
        uint256 eta;
        bool executed;
        bool cancelled;
    }

    mapping(uint256 => Proposal) public proposals;
    mapping(address => Governor) public governors;
    mapping(address => uint256) public votingPower;
    mapping(uint256 => ExecutionQueue) public executionQueue;
    address[] public governorList;

    uint256 public proposalCount;
    uint256 public votingDelay = 1 days;
    uint256 public votingPeriod = 7 days;
    uint256 public executionDelay = 2 days;
    uint256 public proposalThreshold = 1000;
    uint256 public quorumVotes = 5000;
    uint256 public totalVotingPower = 0;
    uint256 public maxGovernors = 20;
    bool public timelockEnabled = true;

    IAuditTrail public auditTrail;

    event ProposalCreated(
        uint256 indexed proposalId,
        address indexed proposer,
        string title,
        uint256 startBlock,
        uint256 endBlock
    );

    event VoteCast(
        uint256 indexed proposalId,
        address indexed voter,
        VoteType support,
        uint256 votes,
        string reason
    );

    event ProposalQueued(uint256 indexed proposalId, uint256 eta);
    event ProposalExecuted(uint256 indexed proposalId);
    event ProposalCancelled(uint256 indexed proposalId);
    event GovernorAdded(
        address indexed governor,
        uint256 votingPower,
        string name
    );
    event GovernorRemoved(address indexed governor);
    event GovernorUpdated(address indexed governor, uint256 newVotingPower);
    event VotingParametersUpdated(
        uint256 votingDelay,
        uint256 votingPeriod,
        uint256 executionDelay
    );

    modifier onlyGovernor() {
        require(governors[msg.sender].isActive, "Not an active governor");
        _;
    }

    function initialize(address admin, address _auditTrail) public initializer {
        __AccessControl_init();
        __Pausable_init();

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(GOVERNOR_ROLE, admin);
        _grantRole(PROPOSER_ROLE, admin);
        _grantRole(EXECUTOR_ROLE, admin);

        auditTrail = IAuditTrail(_auditTrail);

        // Add initial governor
        _addGovernor(admin, 10000, "Genesis Governor", "");
    }

    function addGovernor(
        address governor,
        uint256 _votingPower,
        string calldata name,
        string calldata contact
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(governor != address(0), "Invalid governor address");
        require(!governors[governor].isActive, "Governor already exists");
        require(governorList.length < maxGovernors, "Too many governors");
        require(_votingPower > 0, "Invalid voting power");

        _addGovernor(governor, _votingPower, name, contact);
    }

    function removeGovernor(
        address governor
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(governors[governor].isActive, "Governor not active");
        require(governorList.length > 1, "Cannot remove last governor");

        governors[governor].isActive = false;
        totalVotingPower -= governors[governor].votingPower;
        votingPower[governor] = 0;

        _removeFromGovernorList(governor);

        _revokeRole(GOVERNOR_ROLE, governor);
        _revokeRole(PROPOSER_ROLE, governor);
        _revokeRole(EXECUTOR_ROLE, governor);

        auditTrail.logTransaction(
            keccak256("GOVERNOR_REMOVED"),
            msg.sender,
            0,
            abi.encodePacked(governor)
        );

        emit GovernorRemoved(governor);
    }

    function updateGovernorVotingPower(
        address governor,
        uint256 newVotingPower
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(governors[governor].isActive, "Governor not active");
        require(newVotingPower > 0, "Invalid voting power");

        uint256 oldPower = governors[governor].votingPower;
        governors[governor].votingPower = newVotingPower;
        votingPower[governor] = newVotingPower;

        totalVotingPower = totalVotingPower - oldPower + newVotingPower;

        emit GovernorUpdated(governor, newVotingPower);
    }

    function propose(
        string calldata title,
        string calldata description,
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas
    ) external onlyRole(PROPOSER_ROLE) whenNotPaused returns (uint256) {
        require(
            votingPower[msg.sender] >= proposalThreshold,
            "Below proposal threshold"
        );
        require(
            targets.length == values.length &&
                values.length == calldatas.length,
            "Mismatched arrays"
        );
        require(
            targets.length > 0 && targets.length <= 10,
            "Invalid proposal length"
        );
        require(
            bytes(title).length > 0 && bytes(title).length <= 200,
            "Invalid title"
        );
        require(bytes(description).length > 0, "Description required");

        uint256 proposalId = ++proposalCount;
        _createProposal(
            proposalId,
            title,
            description,
            targets,
            values,
            calldatas
        );
        return proposalId;
    }

    function castVote(
        uint256 proposalId,
        VoteType support,
        string calldata reason
    ) external onlyGovernor {
        _castVote(proposalId, msg.sender, support, reason);
    }

    function queue(uint256 proposalId) external onlyRole(EXECUTOR_ROLE) {
        require(
            state(proposalId) == ProposalState.SUCCEEDED,
            "Proposal not succeeded"
        );
        require(timelockEnabled, "Timelock not enabled");

        uint256 eta = block.timestamp + executionDelay;
        executionQueue[proposalId] = ExecutionQueue({
            proposalId: proposalId,
            eta: eta,
            executed: false,
            cancelled: false
        });

        proposals[proposalId].eta = eta;
        emit ProposalQueued(proposalId, eta);
    }

    function execute(
        uint256 proposalId
    ) external payable onlyRole(EXECUTOR_ROLE) whenNotPaused {
        ProposalState currentState = state(proposalId);
        require(
            currentState == ProposalState.SUCCEEDED,
            "Proposal not succeeded"
        );

        if (timelockEnabled) {
            _validateTimelock(proposalId);
        }

        Proposal storage proposal = proposals[proposalId];
        proposal.executed = true;
        proposal.state = ProposalState.EXECUTED;

        _executeProposalActions(proposal);

        auditTrail.logTransaction(
            keccak256("PROPOSAL_EXECUTED"),
            msg.sender,
            proposalId,
            ""
        );

        emit ProposalExecuted(proposalId);
    }

    function cancel(uint256 proposalId) external {
        ProposalState currentState = state(proposalId);
        Proposal storage proposal = proposals[proposalId];

        require(
            msg.sender == proposal.proposer ||
                hasRole(DEFAULT_ADMIN_ROLE, msg.sender),
            "Only proposer or admin can cancel"
        );

        require(
            currentState == ProposalState.PENDING ||
                currentState == ProposalState.ACTIVE ||
                currentState == ProposalState.SUCCEEDED,
            "Cannot cancel proposal"
        );

        proposal.state = ProposalState.CANCELLED;

        if (timelockEnabled && executionQueue[proposalId].eta != 0) {
            executionQueue[proposalId].cancelled = true;
        }

        auditTrail.logTransaction(
            keccak256("PROPOSAL_CANCELLED"),
            msg.sender,
            proposalId,
            ""
        );

        emit ProposalCancelled(proposalId);
    }

    function state(uint256 proposalId) public view returns (ProposalState) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );

        Proposal storage proposal = proposals[proposalId];

        if (proposal.state == ProposalState.CANCELLED) {
            return ProposalState.CANCELLED;
        }

        if (proposal.executed) {
            return ProposalState.EXECUTED;
        }

        if (block.number <= proposal.startBlock) {
            return ProposalState.PENDING;
        }

        if (block.number <= proposal.endBlock) {
            return ProposalState.ACTIVE;
        }

        if (
            timelockEnabled &&
            proposal.eta != 0 &&
            block.timestamp > proposal.eta + 30 days
        ) {
            return ProposalState.EXPIRED;
        }

        uint256 totalVotes = proposal.forVotes +
            proposal.againstVotes +
            proposal.abstainVotes;

        if (
            totalVotes < quorumVotes ||
            proposal.forVotes <= proposal.againstVotes
        ) {
            return ProposalState.DEFEATED;
        }

        return ProposalState.SUCCEEDED;
    }

    // Internal helper functions
    function _createProposal(
        uint256 proposalId,
        string calldata title,
        string calldata description,
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata calldatas
    ) internal {
        uint256 startBlock = block.number + (votingDelay / 12);
        uint256 endBlock = startBlock + (votingPeriod / 12);

        Proposal storage proposal = proposals[proposalId];
        proposal.id = proposalId;
        proposal.proposer = msg.sender;
        proposal.title = title;
        proposal.description = description;
        proposal.targets = targets;
        proposal.values = values;
        proposal.calldatas = calldatas;
        proposal.startBlock = startBlock;
        proposal.endBlock = endBlock;
        proposal.state = ProposalState.PENDING;

        auditTrail.logTransaction(
            keccak256("PROPOSAL_CREATED"),
            msg.sender,
            proposalId,
            ""
        );

        emit ProposalCreated(
            proposalId,
            msg.sender,
            title,
            startBlock,
            endBlock
        );
    }

    function _castVote(
        uint256 proposalId,
        address voter,
        VoteType support,
        string memory reason
    ) internal {
        require(state(proposalId) == ProposalState.ACTIVE, "Voting not active");

        Proposal storage proposal = proposals[proposalId];
        require(!proposal.hasVoted[voter], "Already voted");

        uint256 votes = votingPower[voter];
        require(votes > 0, "No voting power");

        proposal.hasVoted[voter] = true;
        proposal.votes[voter] = support;
        proposal.votingPower[voter] = votes;

        if (support == VoteType.AGAINST) {
            proposal.againstVotes += votes;
        } else if (support == VoteType.FOR) {
            proposal.forVotes += votes;
        } else {
            proposal.abstainVotes += votes;
        }

        auditTrail.logTransaction(
            keccak256("VOTE_CAST"),
            voter,
            votes,
            abi.encodePacked(proposalId, uint256(support))
        );

        emit VoteCast(proposalId, voter, support, votes, reason);
    }

    function _validateTimelock(uint256 proposalId) internal {
        ExecutionQueue storage queuedProposal = executionQueue[proposalId];
        require(queuedProposal.eta != 0, "Proposal not queued");
        require(block.timestamp >= queuedProposal.eta, "Timelock not expired");
        require(!queuedProposal.executed, "Already executed");
        require(!queuedProposal.cancelled, "Proposal cancelled");

        queuedProposal.executed = true;
    }

    function _executeProposalActions(Proposal storage proposal) internal {
        for (uint256 i = 0; i < proposal.targets.length; i++) {
            (bool success, bytes memory returndata) = proposal.targets[i].call{
                value: proposal.values[i]
            }(proposal.calldatas[i]);

            if (!success) {
                if (returndata.length > 0) {
                    assembly {
                        let returndata_size := mload(returndata)
                        revert(add(32, returndata), returndata_size)
                    }
                } else {
                    revert("Execution failed");
                }
            }
        }
    }

    function _removeFromGovernorList(address governor) internal {
        for (uint256 i = 0; i < governorList.length; i++) {
            if (governorList[i] == governor) {
                governorList[i] = governorList[governorList.length - 1];
                governorList.pop();
                break;
            }
        }
    }

    function _addGovernor(
        address governor,
        uint256 _votingPower,
        string memory name,
        string memory contact
    ) internal {
        governors[governor] = Governor({
            governor: governor,
            votingPower: _votingPower,
            isActive: true,
            joinedAt: block.timestamp,
            name: name,
            contact: contact
        });

        votingPower[governor] = _votingPower;
        totalVotingPower = totalVotingPower + _votingPower;
        governorList.push(governor);

        _grantRole(GOVERNOR_ROLE, governor);
        _grantRole(PROPOSER_ROLE, governor);
        _grantRole(EXECUTOR_ROLE, governor);

        emit GovernorAdded(governor, _votingPower, name);
    }

    // Simple view functions to avoid stack depth issues
    function getProposalId(uint256 proposalId) external view returns (uint256) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        return proposals[proposalId].id;
    }

    function getProposalProposer(
        uint256 proposalId
    ) external view returns (address) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        return proposals[proposalId].proposer;
    }

    function getProposalTitle(
        uint256 proposalId
    ) external view returns (string memory) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        return proposals[proposalId].title;
    }

    function getProposalDescription(
        uint256 proposalId
    ) external view returns (string memory) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        return proposals[proposalId].description;
    }

    function getProposalBlocks(
        uint256 proposalId
    ) external view returns (uint256 startBlock, uint256 endBlock) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        Proposal storage proposal = proposals[proposalId];
        return (proposal.startBlock, proposal.endBlock);
    }

    function getProposalVotes(
        uint256 proposalId
    )
        external
        view
        returns (uint256 forVotes, uint256 againstVotes, uint256 abstainVotes)
    {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        Proposal storage proposal = proposals[proposalId];
        return (
            proposal.forVotes,
            proposal.againstVotes,
            proposal.abstainVotes
        );
    }

    function getProposalState(
        uint256 proposalId
    ) external view returns (ProposalState) {
        return state(proposalId);
    }

    function getProposalExecuted(
        uint256 proposalId
    ) external view returns (bool) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        return proposals[proposalId].executed;
    }

    function getProposalEta(
        uint256 proposalId
    ) external view returns (uint256) {
        require(
            proposalId > 0 && proposalId <= proposalCount,
            "Invalid proposal ID"
        );
        return proposals[proposalId].eta;
    }

    function getProposalActions(
        uint256 proposalId
    )
        external
        view
        returns (
            address[] memory targets,
            uint256[] memory values,
            bytes[] memory calldatas
        )
    {
        Proposal storage proposal = proposals[proposalId];
        return (proposal.targets, proposal.values, proposal.calldatas);
    }

    function getVote(
        uint256 proposalId,
        address voter
    ) external view returns (bool hasVoted, VoteType vote, uint256 votes) {
        Proposal storage proposal = proposals[proposalId];
        return (
            proposal.hasVoted[voter],
            proposal.votes[voter],
            proposal.votingPower[voter]
        );
    }

    function getGovernorInfo(
        address governor
    ) external view returns (Governor memory) {
        return governors[governor];
    }

    function getGovernorsCount() external view returns (uint256) {
        uint256 activeCount = 0;
        for (uint256 i = 0; i < governorList.length; i++) {
            if (governors[governorList[i]].isActive) {
                activeCount++;
            }
        }
        return activeCount;
    }

    function getGovernorAt(uint256 index) external view returns (address) {
        require(index < governorList.length, "Index out of bounds");
        return governorList[index];
    }

    function isGovernorActive(address governor) external view returns (bool) {
        return governors[governor].isActive;
    }

    function canExecute(
        uint256 proposalId
    ) external view returns (bool canExec, string memory reason) {
        ProposalState currentState = state(proposalId);

        if (currentState != ProposalState.SUCCEEDED) {
            return (false, "Proposal not succeeded");
        }

        if (timelockEnabled) {
            ExecutionQueue memory queuedProposal = executionQueue[proposalId];

            if (queuedProposal.eta == 0) {
                return (false, "Proposal not queued");
            }

            if (block.timestamp < queuedProposal.eta) {
                return (false, "Timelock not expired");
            }

            if (queuedProposal.executed) {
                return (false, "Already executed");
            }

            if (queuedProposal.cancelled) {
                return (false, "Proposal cancelled");
            }
        }

        return (true, "");
    }

    function getQuorumReached(
        uint256 proposalId
    )
        external
        view
        returns (bool reached, uint256 totalVotes, uint256 required)
    {
        Proposal storage proposal = proposals[proposalId];
        uint256 total = proposal.forVotes +
            proposal.againstVotes +
            proposal.abstainVotes;
        return (total >= quorumVotes, total, quorumVotes);
    }

    // Admin functions
    function setVotingParameters(
        uint256 _votingDelay,
        uint256 _votingPeriod,
        uint256 _executionDelay,
        uint256 _proposalThreshold,
        uint256 _quorumVotes
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(
            _votingDelay >= 1 hours && _votingDelay <= 7 days,
            "Invalid voting delay"
        );
        require(
            _votingPeriod >= 1 days && _votingPeriod <= 30 days,
            "Invalid voting period"
        );
        require(
            _executionDelay >= 1 hours && _executionDelay <= 30 days,
            "Invalid execution delay"
        );
        require(_proposalThreshold > 0, "Invalid proposal threshold");
        require(_quorumVotes > 0, "Invalid quorum votes");

        votingDelay = _votingDelay;
        votingPeriod = _votingPeriod;
        executionDelay = _executionDelay;
        proposalThreshold = _proposalThreshold;
        quorumVotes = _quorumVotes;

        emit VotingParametersUpdated(
            _votingDelay,
            _votingPeriod,
            _executionDelay
        );
    }

    function setTimelockEnabled(
        bool _enabled
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        timelockEnabled = _enabled;
    }

    function setMaxGovernors(
        uint256 _maxGovernors
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(
            _maxGovernors >= governorList.length,
            "Cannot be less than current governors"
        );
        require(_maxGovernors <= 50, "Too many governors");
        maxGovernors = _maxGovernors;
    }

    function pause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _unpause();
    }

    function emergencyExecute(
        address target,
        uint256 value,
        bytes calldata data
    ) external payable onlyRole(DEFAULT_ADMIN_ROLE) {
        (bool success, ) = target.call{value: value}(data);
        require(success, "Emergency execution failed");

        if (address(auditTrail) != address(0)) {
            auditTrail.logTransaction(
                keccak256("EMERGENCY_EXECUTION"),
                msg.sender,
                0,
                ""
            );
        }
    }

    receive() external payable {
        // Allow contract to receive ETH for proposal execution
    }
}
