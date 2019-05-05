//! Defines blocks and their storage/computation aspects, but no client output.

pub mod universe;
pub use self::universe::Universe;
pub use self::universe::Block;
pub use self::universe::BlockId;
pub use self::universe::BlockState;

pub mod storage;
pub use self::storage::BlockStorage;

pub mod coords;
pub use self::coords::BlockCoord;
