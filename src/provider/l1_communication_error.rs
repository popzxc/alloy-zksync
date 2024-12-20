/// Enum to describe errors that might occur during L1 -> L2 communication.
#[derive(Debug, thiserror::Error)]
pub enum L1CommunicationError {
    #[error("NewPriorityRequest event log was not found in L1 -> L2 transaction.")]
    NewPriorityRequestLogNotFound,
    #[error("Custom L1 -> L2 communication error.")]
    Custom(&'static str),
}
