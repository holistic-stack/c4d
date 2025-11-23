pub mod ds;
pub mod vec3;

// Re-export commonly used types
pub mod half_edge {
    pub use super::ds::{Face, HalfEdge, Vertex};
}
