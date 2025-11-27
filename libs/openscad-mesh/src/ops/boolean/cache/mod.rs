//! # Mesh Cache
//!
//! Caches mesh results for repeated module calls and common operations.
//! Uses content-based hashing for cache keys.
//!
//! ## Features
//!
//! - **Content-based keys**: Hash of geometry parameters
//! - **LRU eviction**: Least recently used eviction when full
//! - **Statistics**: Track hit/miss rates for tuning
//!
//! ## Example
//!
//! ```rust,ignore
//! use openscad_mesh::ops::boolean::cache::MeshCache;
//!
//! let mut cache = MeshCache::new(100);
//! let key = cache.key_for_sphere(5.0, 32);
//! 
//! if let Some(mesh) = cache.get(&key) {
//!     // Cache hit
//! } else {
//!     let mesh = create_sphere(5.0, 32);
//!     cache.put(key, mesh);
//! }
//! ```

use crate::mesh::Mesh;
use std::collections::HashMap;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache key for mesh lookup.
///
/// Content-based hash of geometry parameters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub String);

impl CacheKey {
    /// Creates a new cache key from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Creates a key for a sphere primitive.
    ///
    /// # Arguments
    ///
    /// * `radius` - Sphere radius
    /// * `segments` - Number of segments
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let key = CacheKey::sphere(5.0, 32);
    /// ```
    pub fn sphere(radius: f64, segments: u32) -> Self {
        Self(format!("sphere:r={:.10}:s={}", radius, segments))
    }

    /// Creates a key for a cube primitive.
    ///
    /// # Arguments
    ///
    /// * `size` - Cube dimensions [x, y, z]
    /// * `center` - Whether cube is centered
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let key = CacheKey::cube([10.0, 10.0, 10.0], true);
    /// ```
    pub fn cube(size: [f64; 3], center: bool) -> Self {
        Self(format!(
            "cube:x={:.10}:y={:.10}:z={:.10}:c={}",
            size[0], size[1], size[2], center
        ))
    }

    /// Creates a key for a cylinder primitive.
    ///
    /// # Arguments
    ///
    /// * `height` - Cylinder height
    /// * `r1` - Bottom radius
    /// * `r2` - Top radius
    /// * `center` - Whether centered
    /// * `segments` - Number of segments
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let key = CacheKey::cylinder(10.0, 5.0, 5.0, true, 32);
    /// ```
    pub fn cylinder(height: f64, r1: f64, r2: f64, center: bool, segments: u32) -> Self {
        Self(format!(
            "cyl:h={:.10}:r1={:.10}:r2={:.10}:c={}:s={}",
            height, r1, r2, center, segments
        ))
    }

    /// Creates a key for a module call.
    ///
    /// # Arguments
    ///
    /// * `module_name` - Name of the module
    /// * `args_hash` - Hash of the arguments
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let key = CacheKey::module("my_module", 12345);
    /// ```
    pub fn module(module_name: &str, args_hash: u64) -> Self {
        Self(format!("mod:{}:{}", module_name, args_hash))
    }

    /// Creates a key for a CSG operation result.
    ///
    /// # Arguments
    ///
    /// * `op` - Operation name ("union", "difference", "intersection")
    /// * `left_hash` - Hash of left operand
    /// * `right_hash` - Hash of right operand
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let key = CacheKey::operation("union", hash_a, hash_b);
    /// ```
    pub fn operation(op: &str, left_hash: u64, right_hash: u64) -> Self {
        Self(format!("op:{}:{}:{}", op, left_hash, right_hash))
    }
}

/// Cache entry with metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached mesh
    mesh: Arc<Mesh>,
    /// Access count for LRU
    access_count: u64,
    /// Size estimate (vertex count)
    size: usize,
}

/// Mesh cache with LRU eviction.
///
/// Caches mesh results to avoid recomputation.
///
/// # Example
///
/// ```rust,ignore
/// let mut cache = MeshCache::new(100); // Max 100 entries
/// ```
pub struct MeshCache {
    /// Cached entries
    entries: HashMap<CacheKey, CacheEntry>,
    /// Maximum number of entries
    max_entries: usize,
    /// Total access count for LRU ordering
    total_accesses: u64,
    /// Statistics
    stats: CacheStats,
}

/// Cache statistics for monitoring.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Total entries stored
    pub total_stored: u64,
}

impl CacheStats {
    /// Computes the hit rate (0.0 to 1.0).
    ///
    /// # Returns
    ///
    /// Hit rate as a fraction, or 0.0 if no accesses.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl MeshCache {
    /// Creates a new cache with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `max_entries` - Maximum number of cached meshes
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let cache = MeshCache::new(100);
    /// ```
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_entries,
            total_accesses: 0,
            stats: CacheStats::default(),
        }
    }

    /// Gets a mesh from the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    ///
    /// # Returns
    ///
    /// The cached mesh if found, or None.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(mesh) = cache.get(&key) {
    ///     // Use cached mesh
    /// }
    /// ```
    pub fn get(&mut self, key: &CacheKey) -> Option<Arc<Mesh>> {
        self.total_accesses += 1;

        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_count = self.total_accesses;
            self.stats.hits += 1;
            Some(Arc::clone(&entry.mesh))
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Puts a mesh into the cache.
    ///
    /// Evicts least recently used entries if at capacity.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `mesh` - The mesh to cache
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// cache.put(key, mesh);
    /// ```
    pub fn put(&mut self, key: CacheKey, mesh: Mesh) {
        // Evict if at capacity
        while self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        self.total_accesses += 1;
        let size = mesh.vertex_count();

        self.entries.insert(key, CacheEntry {
            mesh: Arc::new(mesh),
            access_count: self.total_accesses,
            size,
        });

        self.stats.total_stored += 1;
    }

    /// Puts an Arc<Mesh> into the cache (avoids cloning).
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `mesh` - The mesh to cache (already in Arc)
    pub fn put_arc(&mut self, key: CacheKey, mesh: Arc<Mesh>) {
        // Evict if at capacity
        while self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        self.total_accesses += 1;
        let size = mesh.vertex_count();

        self.entries.insert(key, CacheEntry {
            mesh,
            access_count: self.total_accesses,
            size,
        });

        self.stats.total_stored += 1;
    }

    /// Evicts the least recently used entry.
    fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        // Find entry with lowest access count
        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            self.entries.remove(&key);
            self.stats.evictions += 1;
        }
    }

    /// Returns the current number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clears all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns cache statistics.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Computes a hash for arbitrary values.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to hash
    ///
    /// # Returns
    ///
    /// A 64-bit hash.
    pub fn hash_value<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for MeshCache {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 entries
    }
}

/// Global cache for module results.
///
/// Thread-local to avoid synchronization overhead.
#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static GLOBAL_CACHE: std::cell::RefCell<MeshCache> = 
        std::cell::RefCell::new(MeshCache::new(500));
}

/// Gets a mesh from the global cache (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn global_get(key: &CacheKey) -> Option<Arc<Mesh>> {
    GLOBAL_CACHE.with(|cache| cache.borrow_mut().get(key))
}

/// Puts a mesh into the global cache (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn global_put(key: CacheKey, mesh: Mesh) {
    GLOBAL_CACHE.with(|cache| cache.borrow_mut().put(key, mesh));
}

/// Gets global cache stats (native only).
#[cfg(not(target_arch = "wasm32"))]
pub fn global_stats() -> CacheStats {
    GLOBAL_CACHE.with(|cache| cache.borrow().stats().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    fn create_test_mesh() -> Mesh {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));
        mesh.add_triangle(0, 1, 2);
        mesh
    }

    #[test]
    fn test_cache_key_sphere() {
        let key1 = CacheKey::sphere(5.0, 32);
        let key2 = CacheKey::sphere(5.0, 32);
        let key3 = CacheKey::sphere(5.0, 64);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cache_put_get() {
        let mut cache = MeshCache::new(10);
        let key = CacheKey::sphere(5.0, 32);
        let mesh = create_test_mesh();

        cache.put(key.clone(), mesh);

        let result = cache.get(&key);
        assert!(result.is_some());
        assert_eq!(result.unwrap().vertex_count(), 3);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = MeshCache::new(10);
        let key = CacheKey::sphere(5.0, 32);

        let result = cache.get(&key);
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = MeshCache::new(3);

        // Fill cache
        for i in 0..3 {
            let key = CacheKey::sphere(i as f64, 32);
            cache.put(key, create_test_mesh());
        }

        assert_eq!(cache.len(), 3);

        // Add one more, should evict
        let key = CacheKey::sphere(100.0, 32);
        cache.put(key, create_test_mesh());

        assert_eq!(cache.len(), 3);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = MeshCache::new(10);
        let key = CacheKey::sphere(5.0, 32);

        // Miss
        cache.get(&key);
        assert_eq!(cache.stats().misses, 1);
        assert_eq!(cache.stats().hits, 0);

        // Put and hit
        cache.put(key.clone(), create_test_mesh());
        cache.get(&key);
        assert_eq!(cache.stats().hits, 1);

        // Hit rate
        assert!((cache.stats().hit_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = MeshCache::new(10);
        let key = CacheKey::sphere(5.0, 32);
        cache.put(key, create_test_mesh());

        assert_eq!(cache.len(), 1);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }
}
