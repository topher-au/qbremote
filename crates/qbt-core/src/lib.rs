//! Application services, independent of any UI toolkit: profiles and secrets, connection
//! lifecycle, pollers, and the command bus.

pub mod actions;
pub mod errors;
pub mod fmt;
pub mod profile;
pub mod secrets;
pub mod session;
pub mod settings;

pub use profile::{AuthKind, Profile, ProfileStore};
pub use secrets::{KeyringSecrets, MemorySecrets, SecretStore};
pub use session::{
    CommandResult, ConnectionState, Data, Event, Session, SessionHandle, SessionInfo, Watch,
    WatchGuard,
};
pub use settings::ClientSettings;
