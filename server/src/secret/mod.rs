// SPDX-License-Identifier: Apache-2.0

mod secret_store;

#[cfg(test)]
mod mock_secret_store;

pub use secret_store::{SecretStore, SecretStoreError, SecretStorePopResult};

#[cfg(test)]
pub use mock_secret_store::MockSecretStore;
