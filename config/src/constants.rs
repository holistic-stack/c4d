//! # Configuration Constants
//!
//! Centralized constants for the OpenSCAD pipeline. All geometry calculations,
//! tessellation parameters, and precision values are defined here.
//!
//! ## Categories
//!
//! - **Precision**: Floating-point comparison tolerances
//! - **Resolution**: Default tessellation parameters ($fn, $fa, $fs)
//! - **Limits**: Maximum values for safety bounds
//! - **Scaling**: Coordinate system scaling factors

// =============================================================================
// PRECISION CONSTANTS
// =============================================================================

/// Epsilon for floating-point comparisons.
///
/// Used for determining if two floating-point values are "equal" within
/// numerical tolerance. This value is chosen to balance precision with
/// robustness against floating-point errors.
///
/// # Example
///
/// ```rust
/// use config::constants::EPSILON;
///
/// fn approximately_equal(a: f64, b: f64) -> bool {
///     (a - b).abs() < EPSILON
/// }
///
/// assert!(approximately_equal(1.0, 1.0 + 1e-11));
/// ```
pub const EPSILON: f64 = 1e-10;

/// Epsilon for vertex deduplication.
///
/// Slightly larger tolerance used when merging nearly-identical vertices
/// during mesh optimization. This helps clean up numerical noise from
/// boolean operations and transformations.
///
/// # Example
///
/// ```rust
/// use config::constants::VERTEX_MERGE_EPSILON;
///
/// fn vertices_should_merge(v1: [f64; 3], v2: [f64; 3]) -> bool {
///     let dx = v1[0] - v2[0];
///     let dy = v1[1] - v2[1];
///     let dz = v1[2] - v2[2];
///     (dx * dx + dy * dy + dz * dz).sqrt() < VERTEX_MERGE_EPSILON
/// }
/// ```
pub const VERTEX_MERGE_EPSILON: f64 = 1e-8;

/// Scaling factor for converting f64 coordinates to i64 for integer algorithms.
///
/// Used when algorithms require integer arithmetic (e.g., some polygon
/// clipping algorithms). Coordinates are multiplied by this factor before
/// conversion to i64, then divided after the operation.
///
/// # Example
///
/// ```rust
/// use config::constants::COORDINATE_SCALE;
///
/// fn to_integer_coord(value: f64) -> i64 {
///     (value * COORDINATE_SCALE) as i64
/// }
///
/// fn from_integer_coord(value: i64) -> f64 {
///     value as f64 / COORDINATE_SCALE
/// }
/// ```
pub const COORDINATE_SCALE: f64 = 1e6;

// =============================================================================
// RESOLUTION CONSTANTS (OpenSCAD $fn, $fa, $fs)
// =============================================================================

/// Default value for $fn (fragment count override).
///
/// When $fn > 0, it specifies the exact number of fragments for circular
/// shapes. When $fn = 0, the fragment count is calculated from $fa and $fs.
///
/// OpenSCAD default: 0 (use $fa/$fs calculation)
///
/// # Example
///
/// ```rust
/// use config::constants::DEFAULT_FN;
///
/// let user_fn: Option<f64> = None;
/// let fn_value = user_fn.unwrap_or(DEFAULT_FN);
/// assert_eq!(fn_value, 0.0);
/// ```
pub const DEFAULT_FN: f64 = 0.0;

/// Default value for $fa (minimum fragment angle in degrees).
///
/// The minimum angle between consecutive fragments. Smaller values produce
/// smoother circles but more triangles.
///
/// OpenSCAD default: 12.0 degrees
///
/// # Example
///
/// ```rust
/// use config::constants::DEFAULT_FA;
///
/// // Maximum fragments from angle: 360 / $fa
/// let max_from_angle = 360.0 / DEFAULT_FA; // = 30 fragments
/// ```
pub const DEFAULT_FA: f64 = 12.0;

/// Default value for $fs (minimum fragment size).
///
/// The minimum length of a fragment edge. Smaller values produce smoother
/// circles for larger radii.
///
/// OpenSCAD default: 2.0 units
///
/// # Example
///
/// ```rust
/// use config::constants::DEFAULT_FS;
///
/// // Maximum fragments from size: circumference / $fs = 2*PI*r / $fs
/// let radius = 10.0;
/// let max_from_size = (2.0 * std::f64::consts::PI * radius) / DEFAULT_FS;
/// ```
pub const DEFAULT_FS: f64 = 2.0;

/// Minimum number of fragments for any circular shape.
///
/// OpenSCAD enforces a minimum of 3 fragments (triangle) for circles,
/// but 5 is used as a practical minimum for better visual quality.
///
/// # Example
///
/// ```rust
/// use config::constants::MIN_FRAGMENTS;
///
/// let computed_fragments = 2; // Too few
/// let actual_fragments = computed_fragments.max(MIN_FRAGMENTS);
/// ```
pub const MIN_FRAGMENTS: u32 = 5;

/// Maximum number of fragments for any circular shape.
///
/// Safety limit to prevent excessive tessellation that could cause
/// memory issues or slow rendering.
///
/// # Example
///
/// ```rust
/// use config::constants::MAX_FRAGMENTS;
///
/// let computed_fragments = 10000; // Too many
/// let actual_fragments = computed_fragments.min(MAX_FRAGMENTS);
/// ```
pub const MAX_FRAGMENTS: u32 = 1000;

// =============================================================================
// LIMIT CONSTANTS
// =============================================================================

/// Maximum recursion depth for evaluator.
///
/// Prevents stack overflow from deeply nested or recursive OpenSCAD code.
/// The `stacker` crate is used to handle deep recursion safely.
///
/// # Example
///
/// ```rust
/// use config::constants::MAX_RECURSION_DEPTH;
///
/// let current_depth = 500;
/// assert!(current_depth < MAX_RECURSION_DEPTH);
/// ```
pub const MAX_RECURSION_DEPTH: usize = 1000;

/// Maximum number of vertices in a single mesh.
///
/// Safety limit to prevent memory exhaustion from extremely complex models.
///
/// # Example
///
/// ```rust
/// use config::constants::MAX_VERTICES;
///
/// let vertex_count = 1000;
/// assert!(vertex_count < MAX_VERTICES);
/// ```
pub const MAX_VERTICES: usize = 10_000_000;

/// Maximum number of triangles in a single mesh.
///
/// Safety limit to prevent memory exhaustion from extremely complex models.
pub const MAX_TRIANGLES: usize = 10_000_000;

/// Maximum file size for imported files (in bytes).
///
/// Prevents loading extremely large files that could cause memory issues.
/// 100 MB default.
pub const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

// =============================================================================
// GEOMETRY CONSTANTS
// =============================================================================

/// Default convexity value for extrusions and other operations.
///
/// Convexity is a hint for rendering engines about the maximum number of
/// front-facing surfaces a ray might intersect. Higher values are needed
/// for more complex shapes.
///
/// # Example
///
/// ```rust
/// use config::constants::DEFAULT_CONVEXITY;
///
/// let user_convexity: Option<u32> = None;
/// let convexity = user_convexity.unwrap_or(DEFAULT_CONVEXITY);
/// assert_eq!(convexity, 1);
/// ```
pub const DEFAULT_CONVEXITY: u32 = 1;

/// Default number of slices for linear_extrude with twist.
///
/// When twist is applied, the extrusion is divided into this many slices
/// to approximate the spiral shape.
pub const DEFAULT_EXTRUDE_SLICES: u32 = 1;

// =============================================================================
// COLOR CONSTANTS
// =============================================================================

/// Default color when none is specified (light gray).
///
/// RGBA values in range [0.0, 1.0].
pub const DEFAULT_COLOR: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Computes the number of fragments for a circular shape.
///
/// Implements OpenSCAD's resolution formula:
/// - If $fn > 0: use $fn (clamped to MIN_FRAGMENTS..MAX_FRAGMENTS)
/// - Otherwise: ceil(min(360/$fa, 2*PI*r/$fs)) clamped to MIN_FRAGMENTS..MAX_FRAGMENTS
///
/// # Arguments
///
/// * `radius` - The radius of the circular shape
/// * `fn_value` - The $fn override (0 means use $fa/$fs)
/// * `fa_value` - The $fa value (minimum angle per fragment)
/// * `fs_value` - The $fs value (minimum fragment size)
///
/// # Returns
///
/// The number of fragments to use for tessellation.
///
/// # Example
///
/// ```rust
/// use config::constants::{compute_fragments, DEFAULT_FN, DEFAULT_FA, DEFAULT_FS};
///
/// // With $fn override
/// let fragments = compute_fragments(10.0, 32.0, DEFAULT_FA, DEFAULT_FS);
/// assert_eq!(fragments, 32);
///
/// // Without $fn override (uses $fa/$fs)
/// let fragments = compute_fragments(10.0, 0.0, DEFAULT_FA, DEFAULT_FS);
/// assert!(fragments >= 5); // At least MIN_FRAGMENTS
/// ```
pub fn compute_fragments(radius: f64, fn_value: f64, fa_value: f64, fs_value: f64) -> u32 {
    let fragments = if fn_value > 0.0 {
        // Use $fn directly
        fn_value as u32
    } else {
        // Calculate from $fa and $fs
        let from_angle = 360.0 / fa_value;
        let from_size = (2.0 * std::f64::consts::PI * radius) / fs_value;
        from_angle.min(from_size).ceil() as u32
    };

    // Clamp to valid range
    fragments.clamp(MIN_FRAGMENTS, MAX_FRAGMENTS)
}

/// Checks if two f64 values are approximately equal within EPSILON.
///
/// # Example
///
/// ```rust
/// use config::constants::approx_equal;
///
/// assert!(approx_equal(1.0, 1.0 + 1e-11));
/// assert!(!approx_equal(1.0, 1.1));
/// ```
#[inline]
pub fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

/// Checks if a f64 value is approximately zero within EPSILON.
///
/// # Example
///
/// ```rust
/// use config::constants::approx_zero;
///
/// assert!(approx_zero(1e-11));
/// assert!(!approx_zero(0.1));
/// ```
#[inline]
pub fn approx_zero(value: f64) -> bool {
    value.abs() < EPSILON
}
