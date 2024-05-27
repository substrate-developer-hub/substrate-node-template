use crate::{mock::*, Error};
use frame_support::{assert_err, assert_ok};

#[test]
fn vote_for_cdr_not_found() {
	new_test_ext().execute_with(|| {
		let ipfs = H256::zero();
		let pv_info = PVoPOInfo {
			documentation_ipfs: BoundedString::<IPFSLength>::truncate_from("ipfs_link"),
			penalty_level: 0,
			penalty_timeout: 0,
		};

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Create project validator
		ProjectValidators::<Test>::insert(bob(), pv_info);

		assert_err!(
			Veles::cast_vote(RuntimeOrigin::signed(bob()), VoteType::CdrVote, ipfs, false),
			Error::<Test>::ReportNotFound
		);
	});
}

#[test]
fn vote_for_project_proposal_not_found() {
	new_test_ext().execute_with(|| {
		let ipfs = H256::zero();
		let pv_info = PVoPOInfo {
			documentation_ipfs: BoundedString::<IPFSLength>::truncate_from("ipfs_link"),
			penalty_level: 0,
			penalty_timeout: 0,
		};

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Create project validator
		ProjectValidators::<Test>::insert(bob(), pv_info);

		assert_err!(
			Veles::cast_vote(RuntimeOrigin::signed(bob()), VoteType::ProposalVote, ipfs, false),
			Error::<Test>::ProjectProposalNotFound
		);
	});
}

#[test]
fn vote_for_project_proposal_ok() {
	new_test_ext().execute_with(|| {
		let ipfs = H256::zero();
		let pv_info = PVoPOInfo {
			documentation_ipfs: BoundedString::<IPFSLength>::truncate_from("ipfs_link"),
			penalty_level: 0,
			penalty_timeout: 0,
		};
		let project_proposal_info = PProposalInfo {
			project_owner: bob(),
			creation_date: 0,
			project_hash: ipfs,
			votes_for: BTreeSet::<AccountId>::new(),
			votes_against: BTreeSet::<AccountId>::new(),
		};

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Create project proposal
		ProjectProposals::<Test>::insert(ipfs, project_proposal_info);
		// Create project validator
		ProjectValidators::<Test>::insert(bob(), pv_info);

		assert_ok!(Veles::cast_vote(
			RuntimeOrigin::signed(bob()),
			VoteType::ProposalVote,
			ipfs,
			false
		));
	});
}

#[test]
fn vote_for_cdr_ok() {
	new_test_ext().execute_with(|| {
		let ipfs = H256::zero();
		let pv_info = PVoPOInfo {
			documentation_ipfs: BoundedString::<IPFSLength>::truncate_from("ipfs_link"),
			penalty_level: 0,
			penalty_timeout: 0,
		};
		let cdr_info = CFReportInfo {
			account_id: alice(),
			creation_date: 0,
			carbon_deficit: 0,
			votes_for: BTreeSet::<AccountId>::new(),
			votes_against: BTreeSet::<AccountId>::new(),
		};

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Create project proposal
		CarbonDeficitReports::<Test>::insert(ipfs, cdr_info);
		// Create project validator
		ProjectValidators::<Test>::insert(bob(), pv_info);

		assert_ok!(Veles::cast_vote(RuntimeOrigin::signed(bob()), VoteType::CdrVote, ipfs, false));
	});
}

#[test]
fn project_proposal_ok() {
	new_test_ext().execute_with(|| {
		let ipfs = H256::zero();
		let pv_po_info = PVoPOInfo {
			documentation_ipfs: BoundedString::<IPFSLength>::truncate_from("ipfs_link"),
			penalty_level: 0,
			penalty_timeout: 0,
		};
		let project_proposal_info = PProposalInfo {
			project_owner: bob(),
			creation_date: 0,
			project_hash: ipfs,
			votes_for: BTreeSet::<AccountId>::new(),
			votes_against: BTreeSet::<AccountId>::new(),
		};

		// Go past genesis block so events get deposited
		System::set_block_number(1);

		ProjectOwners::<Test>::insert(bob(), pv_po_info);

		// Create project proposal
		assert_ok!(Veles::propose_project(RuntimeOrigin::signed(bob()), ipfs));

		// Assert project proposal owner account equal to project_owner account
		assert_eq!(bob(), ProjectProposals::<Test>::get(ipfs).unwrap().project_owner);
	});
}

#[test]
fn project_proposal_not_authorized() {
	new_test_ext().execute_with(|| {
		let ipfs = H256::zero();
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Create project proposal
		assert_err!(
			Veles::propose_project(RuntimeOrigin::signed(alice()), ipfs),
			Error::<Test>::NotAuthorized
		);
	});
}
