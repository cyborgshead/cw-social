use cosmwasm_std::{StdError, Uint64};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Deleted cyberlink: {fid}")]
    DeletedCyberlink { fid: String },

    #[error("Not found: {fid}")]
    NotFound { fid: String },

    // TODO: revisit and change to id: String
    #[error("Particular links is not allowed - from: {from}, to: {to}, type: {type_}")]
    InvalidCyberlink {from: String, to: String, type_: String},

    #[error("Type not exists: {type_}")]
    TypeNotExists { type_: String },

    #[error("From not exists: {from}")]
    FromNotExists { from: String },

    #[error("To not exists: {to}")]
    ToNotExists { to: String },

    #[error("Type conflict: link type '{type_}' connecting from '{from}' to '{to}'. Expected type: '{expected_type}' (constraints from: '{expected_from}', to: '{expected_to}'). Received type: '{received_type}' (actual from: '{received_from}', to: '{received_to}')")]
    TypeConflict {
        type_: String,
        from: String,
        to: String,
        expected_type: String,
        expected_from: String,
        expected_to: String,
        received_type: String,
        received_from: String,
        received_to: String,
    },

    #[error("Cannot change cyberlink type: ID {id} from {original_type} to {new_type}")]
    CannotChangeType { id: String, original_type: String, new_type: String },

    #[error("Cannot change cyberlink {field}: ID {id} from {original} to {new}")]
    CannotChangeLinks { id: String, field: String, original: String, new: String },

    #[error("Invalid name format: '{name}' contains a colon character (:) which is not allowed")]
    InvalidNameFormat { name: String },

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Cannot migrate from unsupported version: {previous_version}")]
    CannotMigrateVersion { previous_version: String },

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Invalid link specification: Exactly one of link_from_existing_id or link_to_existing_id must be provided")]
    InvalidLinkSpecification {},
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
