pub mod types;
pub mod detector;
pub mod ai_engine;

pub use types::*;
pub use detector::detect_npu;
pub use ai_engine::AiStreamOptimizer;
