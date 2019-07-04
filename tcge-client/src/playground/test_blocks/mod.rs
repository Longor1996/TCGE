use common::current_time_nanos;
use common::resources;

use blocks;
use blocks::BlockDim;
use blocks::BlockCoord;
use blocks::BlockRaycast;
use blocks::BlocksRef;
use blocks::Block;
use blocks::BlockId;
use blocks::BlockState;

pub const CHUNK_SIZE_BITS: isize = 5;
pub const CHUNK_SIZE: usize = 1 << CHUNK_SIZE_BITS;
pub const CHUNK_SIZE_MASK: usize = CHUNK_SIZE-1;
pub const CHUNK_SLICE: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const CHUNK_SIZE_I: BlockDim = CHUNK_SIZE as BlockDim;
pub const CHUNK_SIZE_MASK_I: BlockDim = CHUNK_SIZE_MASK as BlockDim;
pub const CHUNK_SLICE_I: BlockDim = CHUNK_SLICE as BlockDim;

pub mod chunk_coord;
pub use chunk_coord::ChunkDim;
pub use chunk_coord::ChunkCoord;
use rustc_hash::FxHashMap;

pub mod chunk;
pub use chunk::Chunk;

pub mod chunk_storage;
pub use chunk_storage::*;

pub mod chunk_render;
pub use chunk_render::*;

pub mod chunk_mesher;
pub use chunk_mesher::*;

pub mod block_material;
pub use block_material::*;

pub mod block_bakery;
pub use block_bakery::*;
