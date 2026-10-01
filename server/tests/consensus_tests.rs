//! # Unit tests for MAGI System consensus engine

use magi_server::{
    calculate_verdict, VoteRecord, NODE_BALTHASAR, NODE_CASPER, NODE_MELCHIOR,
    VERDICT_APPROVED_MAJORITY, VERDICT_APPROVED_UNANIMOUS, VERDICT_REJECTED_MAJORITY,
    VERDICT_REJECTED_UNANIMOUS, VERDICT_SPLIT_DECISION, VERDICT_VETO_BALTHASAR, VOTE_APPROVE,
    VOTE_NEUTRAL, VOTE_REJECT,
};

#[test]
fn test_unanimous_approve() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 3,
        },
        VoteRecord {
            node_id: NODE_CASPER.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 1,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_APPROVED_UNANIMOUS);
    assert_eq!(approves, 3);
    assert_eq!(rejects, 0);
    assert_eq!(neutrals, 0);
    assert!(!veto);
}

#[test]
fn test_majority_approve_without_veto() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 7, // Below veto threshold of 8
        },
        VoteRecord {
            node_id: NODE_CASPER.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 3,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_APPROVED_MAJORITY);
    assert_eq!(approves, 2);
    assert_eq!(rejects, 1);
    assert_eq!(neutrals, 0);
    assert!(!veto);
}

#[test]
fn test_balthasar_security_veto_overrides_approves() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 1,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 8, // Triggers veto
        },
        VoteRecord {
            node_id: NODE_CASPER.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_VETO_BALTHASAR);
    assert_eq!(approves, 2);
    assert_eq!(rejects, 1);
    assert_eq!(neutrals, 0);
    assert!(veto);
}

#[test]
fn test_majority_reject() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 6,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 3,
        },
        VoteRecord {
            node_id: NODE_CASPER.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 5,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_REJECTED_MAJORITY);
    assert_eq!(approves, 1);
    assert_eq!(rejects, 2);
    assert_eq!(neutrals, 0);
    assert!(!veto);
}

#[test]
fn test_unanimous_reject() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 7,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 7, // Not a veto because risk < 8
        },
        VoteRecord {
            node_id: NODE_CASPER.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 6,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_REJECTED_UNANIMOUS);
    assert_eq!(approves, 0);
    assert_eq!(rejects, 3);
    assert_eq!(neutrals, 0);
    assert!(!veto);
}

#[test]
fn test_split_decision_with_neutrals() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_NEUTRAL.to_string(),
            risk_score: 4,
        },
        VoteRecord {
            node_id: NODE_CASPER.to_string(),
            vote: VOTE_REJECT.to_string(),
            risk_score: 5,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_SPLIT_DECISION);
    assert_eq!(approves, 1);
    assert_eq!(rejects, 1);
    assert_eq!(neutrals, 1);
    assert!(!veto);
}

#[test]
fn test_unknown_node_cannot_become_consensus_member() {
    let votes = vec![
        VoteRecord {
            node_id: NODE_MELCHIOR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
        VoteRecord {
            node_id: NODE_BALTHASAR.to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
        VoteRecord {
            node_id: "Imposter-4".to_string(),
            vote: VOTE_APPROVE.to_string(),
            risk_score: 2,
        },
    ];

    let (verdict, approves, rejects, neutrals, veto) = calculate_verdict(&votes);
    assert_eq!(verdict, VERDICT_APPROVED_MAJORITY);
    assert_eq!(approves, 2);
    assert_eq!(rejects, 0);
    assert_eq!(neutrals, 0);
    assert!(!veto);
}
