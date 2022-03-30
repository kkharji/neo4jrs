use crate::{Query, Result, RowStream};
use async_trait::async_trait;

#[async_trait]
// Trait to enable both Graph and Txn to be passed into functions
pub trait Execute {
    /// Runs a query using a connection from the connection pool, it doesn't return any
    /// [`RowStream`] as the `run` abstraction discards any stream.
    ///
    /// Use [`Execute::run`] for cases where you just want a write operation
    ///
    /// use [`Execute::execute`] when you are interested in the result stream
    async fn run(&self, q: Query) -> Result<()>;

    /// Executes a query and returns a [`RowStream`]
    async fn execute(&self, q: Query) -> Result<RowStream>;
}
