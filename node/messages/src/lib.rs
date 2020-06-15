// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Message types for the overseer and subsystems.
//!
//! These messages are intended to define the protocol by which different subsystems communicate with each
//! other and signals that they receive from an overseer to coordinate their work.
//! This is intended for use with the `polkadot-overseer` crate.
//!
//! Subsystems' APIs are defined separately from their implementation, leading to easier mocking.

use futures::channel::{mpsc, oneshot};

use polkadot_primitives::Hash;
use polkadot_primitives::parachain::{AbridgedCandidateReceipt, PoVBlock};
use polkadot_node_primitives::SignedStatement;

/// Signals sent by an overseer to a subsystem.
#[derive(PartialEq, Clone, Debug)]
pub enum OverseerSignal {
	/// `Subsystem` should start working on block-based work, given by the relay-chain block hash.
	StartWork(Hash),
	/// `Subsystem` should stop working on block-based work specified by the relay-chain block hash.
	StopWork(Hash),
	/// Conclude the work of the `Overseer` and all `Subsystem`s.
	Conclude,
}

/// A notification of a new backed candidate.
#[derive(Debug)]
pub struct NewBackedCandidate; // TODO [now]

/// Messages received by the Candidate Selection subsystem.
#[derive(Debug)]
pub enum CandidateSelectionSubsystemMessage {
	/// We recommended a particular candidate to be seconded, but it was invalid; penalize the collator.
	/// The hash is the relay parent.
	Invalid(Hash, AbridgedCandidateReceipt),
}

/// Messages received by the Candidate Backing subsystem.
#[derive(Debug)]
pub enum CandidateBackingSubsystemMessage {
	/// Registers a stream listener for updates to the set of backable candidates that could be backed
	/// in a child of the given relay-parent, referenced by its hash.
	RegisterBackingWatcher(Hash, mpsc::Sender<NewBackedCandidate>),
	/// Note that the Candidate Backing subsystem should second the given candidate in the context of the
	/// given relay-parent (ref. by hash). This candidate must be validated.
	Second(Hash, AbridgedCandidateReceipt),
	/// Note a validator's statement about a particular candidate. Disagreements about validity must be escalated
	/// to a broader check by Misbehavior Arbitration. Agreements are simply tallied until a quorum is reached.
	Statement(Hash, SignedStatement),
}

/// Blanket error for validation failing.
#[derive(Debug)]
pub struct ValidationFailed;

/// Messages received by the Validation subsystem
#[derive(Debug)]
pub enum ValidationSubsystemMessage {
	/// Validate a candidate, sending a side-channel response of valid or invalid.
	///
	/// Provide the relay-parent in whose context this should be validated, the full candidate receipt,
	/// and the PoV.
	Validate(
		Hash,
		AbridgedCandidateReceipt,
		PoVBlock,
		oneshot::Sender<Result<(), ValidationFailed>>,
	),
}

/// A message type tying together all message types that are used across Subsystems.
#[derive(Debug)]
pub enum AllMessages {
	/// Message for the validation subsystem.
	Validation(ValidationSubsystemMessage),
	/// Message for the candidate backing subsystem.
	CandidateBacking(CandidateBackingSubsystemMessage),
}

/// A message type that a subsystem receives from an overseer.
/// It wraps signals from an overseer and messages that are circulating
/// between subsystems.
///
/// It is generic over over the message type `M` that a particular `Subsystem` may use.
#[derive(Debug)]
pub enum FromOverseer<M: std::fmt::Debug> {
	/// Signal from the `Overseer`.
	Signal(OverseerSignal),

	/// Some other `Subsystem`'s message.
	Communication {
		msg: M,
	},
}
