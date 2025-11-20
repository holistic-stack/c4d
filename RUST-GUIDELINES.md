# Rust Guidelines 2025

> Comprehensive development guidelines for modern Rust applications with TDD, SOLID principles, and monorepo best practices.

## Table of Contents
- [Core Principles](#core-principles)
- [Monorepo Structure](#monorepo-structure)
- [TDD Approach](#tdd-approach)
- [SOLID Implementation](#solid-implementation)
- [Code Organization](#code-organization)
- [Configuration Management](#configuration-management)
- [Testing Strategies](#testing-strategies)
- [Performance Guidelines](#performance-guidelines)
- [Common Pitfalls](#common-pitfalls)
- [Development Workflow](#development-workflow)

---

## Core Principles

### 🎯 **Do's**
- **Write small, focused functions** (<50 lines) following Single Responsibility Principle
- **Prioritize readability** over clever code
- **Use DRY and KISS principles** consistently
- **Follow TDD with small, incremental changes**
- **Keep files under 500 lines** - split when they grow larger
- **Test real behavior, not implementation details**
- **Use kebab-case for filenames** and snake_case for identifiers
- **Centralize global variables** in configuration files
- **Apply algorithm improvements** through iterative refinement
- **Document changes and project progress** continuously
- **Clean up old documentation** to maintain clarity

### ❌ **Don'ts**
- **Use mocks except for external I/O services**
- **Create files over 500 lines** without proper splitting
- **Write clever code that sacrifices readability**
- **Duplicate logic** (DRY principle violations)
- **Use any types in TypeScript** (strict typing required)
- **Ignore SOLID principles** for convenience
- **Commit large, monolithic changes**
- **Mix concerns in single functions/modules**
- **Use global mutable state** except in controlled patterns
- **Write tests that mock internal behavior**

---

## Monorepo Structure

### 📁 **Recommended Project Layout**

```
project-root/
├── Cargo.toml                 # Workspace configuration
├── .gitignore
├── README.md
├── config/                     # Centralized configuration
│   ├── constants.rs           # Global constants
│   ├── app-config.rs          # Application configuration
│   └── environment/           # Environment-specific configs
│       ├── development.toml
│       ├── staging.toml
│       └── production.toml
├── libs/                       # Reusable libraries (SRP focused)
│   ├── core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models.rs
│   │   │   └── tests/
│   │   │       └── integration.rs
│   │   └── Cargo.toml
│   ├── parser/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── syntax.rs
│   │   │   └── tests/
│   │   │       └── parser-tests.rs
│   │   └── Cargo.toml
│   ├── geometry/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mesh.rs
│   │   │   ├── primitives/
│   │   │   │   ├── cube.rs
│   │   │   │   └── sphere.rs
│   │   │   └── tests/
│   │   │       ├── cube-tests.rs
│   │   │       └── sphere-tests.rs
│   │   └── Cargo.toml
│   └── utils/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── validation.rs
│       │   └── tests/
│       │       └── validation-tests.rs
│       └── Cargo.toml
├── apps/                       # Application binaries
│   ├── cli-tool/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── process.rs
│   │   │   │   └── validate.rs
│   │   │   └── tests/
│   │   │       └── cli-tests.rs
│   │   └── Cargo.toml
│   └── web-api/
│       ├── src/
│       │   ├── main.rs
│       │   ├── handlers/
│       │   └── tests/
│       └── Cargo.toml
├── tests/                      # Cross-crate integration tests
│   ├── integration-tests.rs
│   ├── end-to-end-tests.rs
│   └── performance-tests.rs
├── tools/                      # Development tools and scripts
│   ├── build-wasm.js
│   └── release-helper.rs
└── docs/                       # Documentation
    ├── api/
    └── guides/
```

### 🔧 **Workspace Configuration**

**Root Cargo.toml:**
```toml
[workspace]
resolver = "2"
members = [
    "libs/*",
    "apps/*"
]

[workspace.dependencies]
# Shared dependencies with version constraints
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt-multi-thread"] }
clap = { version = "4.0", features = ["derive"] }

[profile.dev]
opt-level = 1
debug = true
incremental = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

**Individual Crate Structure:**
```toml
# libs/core/Cargo.toml
[package]
name = "project-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# Use workspace dependencies for consistency
thiserror = { workspace = true }
serde = { workspace = true }

[dev-dependencies]
insta = "1.38"
proptest = "1.4"
```

---

## TDD Approach

### 🔄 **TDD Workflow Without Mocks**

1. **Red**: Write a failing test that describes desired behavior
2. **Green**: Implement minimal code to make test pass
3. **Refactor**: Improve code while maintaining test passing
4. **Property Tests**: Add proptest for invariants
5. **Integration**: Verify components work together

### 📝 **Testing Philosophy**

**State-based over interaction-based:**
```rust
// ✅ Test behavior and results
#[test]
fn test_transformation_preserves_data() {
    let input = create_test_data();
    let result = transform(&input);

    assert_eq!(result.count, input.count);
    assert!(result.is_valid());
}

// ❌ Avoid testing internal calls with mocks
#[test]
#[ignore] // Don't do this
fn test_calls_internal_methods() {
    // Tests implementation details, fragile
}
```

### 🧪 **Integration Testing Examples**

```rust
// tests/integration-tests.rs
use project_core::DataProcessor;
use tempfile::TempDir;

#[test]
fn test_full_processing_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.json");

    // Create real test data
    create_test_file(&input_path);

    // Test real file I/O
    let processor = DataProcessor::new();
    processor.process_file(&input_path, &output_path).unwrap();

    // Verify real output
    let output = std::fs::read_to_string(&output_path).unwrap();
    assert!(output.contains("processed"));
}
```

### 🎲 **Property-Based Testing**

```rust
// libs/geometry/src/tests/sphere-tests.rs
use proptest::prelude::*;
use crate::sphere::Sphere;

proptest! {
    #[test]
    fn sphere_volume_positive(radius in 0.1f64..1000.0) {
        let sphere = Sphere::new(radius);
        let volume = sphere.volume();

        prop_assert!(volume > 0.0);
        // Volume should scale with radius cubed
        let volume_ratio = volume / (radius.powi(3));
        prop_assert!(volume_ratio > 3.0 && volume_ratio < 5.0); // 4π/3 ≈ 4.188
    }
}
```

### 📸 **Snapshot Testing**

```rust
// libs/parser/src/tests/parser-tests.rs
use insta::assert_snapshot;
use crate::Parser;

#[test]
fn test_error_message_formatting() {
    let parser = Parser::new();
    let result = parser.parse("invalid input {");

    assert_snapshot!(result.unwrap_err().to_string());
}
```

---

## SOLID Implementation

### 🔸 **Single Responsibility Principle (SRP)**

Each function and module has one reason to change:

```rust
// libs/validation/src/validation.rs
/// Validates user input data
pub mod validation {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ValidationError {
        #[error("Invalid email format: {0}")]
        Email(String),

        #[error("Password too short: minimum {min} characters")]
        PasswordTooShort { min: usize },
    }

    /// Validates email format - single responsibility
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if !email.contains('@') {
            return Err(ValidationError::Email(email.to_string()));
        }
        Ok(())
    }

    /// Validates password requirements - single responsibility
    pub fn validate_password(password: &str) -> Result<(), ValidationError> {
        if password.len() < 8 {
            return Err(ValidationError::PasswordTooShort { min: 8 });
        }
        Ok(())
    }
}
```

### 🔸 **Open/Closed Principle**

```rust
// libs/geometry/src/primitives/mod.rs
/// Extensible primitive system
pub trait Primitive {
    fn volume(&self) -> f64;
    fn surface_area(&self) -> f64;
    fn transform(&self, matrix: &Matrix4) -> Box<dyn Primitive>;
}

// Open for extension, closed for modification
pub struct Cube {
    size: Vector3,
}

impl Primitive for Cube {
    fn volume(&self) -> f64 {
        self.size.x * self.size.y * self.size.z
    }

    fn surface_area(&self) -> f64 {
        2.0 * (
            self.size.x * self.size.y +
            self.size.y * self.size.z +
            self.size.z * self.size.x
        )
    }

    fn transform(&self, matrix: &Matrix4) -> Box<dyn Primitive> {
        Box::new(Cube {
            size: matrix.transform_vector3(&self.size),
        })
    }
}
```

### 🔸 **Dependency Inversion**

```rust
// libs/core/src/repositories/mod.rs
/// Abstraction for data storage
pub trait DataRepository {
    fn save(&self, data: &Data) -> Result<(), RepositoryError>;
    fn load(&self, id: &str) -> Result<Data, RepositoryError>;
}

// Concrete implementation
pub struct FileSystemRepository {
    base_path: PathBuf,
}

impl DataRepository for FileSystemRepository {
    fn save(&self, data: &Data) -> Result<(), RepositoryError> {
        // Implementation details
        Ok(())
    }

    fn load(&self, id: &str) -> Result<Data, RepositoryError> {
        // Implementation details
        Ok(())
    }
}

// Service depends on abstraction, not concrete type
pub struct DataService<R: DataRepository> {
    repository: R,
}

impl<R: DataRepository> DataService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn process_and_save(&self, data: Data) -> Result<(), ServiceError> {
        let processed = self.process(data);
        self.repository.save(&processed)?;
        Ok(())
    }
}
```

---

## Code Organization

### 📂 **File Naming Conventions**

- **Filenames**: kebab-case (e.g., `user-service.rs`, `data-validator.rs`)
- **Module names**: snake_case (e.g., `user_service`, `data_validator`)
- **Struct/Function names**: PascalCase for types, snake_case for functions
- **Test files**: `{module}-tests.rs` (e.g., `user-service-tests.rs`)

### 🗂️ **Module Organization**

```rust
// libs/user-management/src/lib.rs
pub mod models;           // Data structures
pub mod services;         // Business logic
pub mod repositories;     // Data access
pub mod validation;       // Input validation
pub mod utils;           // Utilities

// Re-export public API
pub use models::{User, UserProfile};
pub use services::{UserService, AuthenticationService};
```

### 📄 **File Size Management**

**Split when file exceeds 500 lines:**

```rust
// Before: user-service.rs (600+ lines)
// Split into:

// user-service/mod.rs
pub mod authentication;
pub mod profile_management;
pub mod user_creation;

// user-service/authentication.rs
pub struct AuthenticationService { /* ... */ }

// user-service/profile-management.rs
pub struct ProfileService { /* ... */ }

// user-service/user-creation.rs
pub struct UserCreationService { /* ... */ }
```

---

## Configuration Management

### ⚙️ **Centralized Constants**

```rust
// config/constants.rs
use std::time::Duration;

/// Application-wide constants
pub mod constants {
    /// Default page size for pagination
    pub const DEFAULT_PAGE_SIZE: usize = 20;

    /// Maximum file upload size (10MB)
    pub const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024;

    /// Session timeout duration
    pub const SESSION_TIMEOUT: Duration = Duration::from_secs(3600);

    /// API rate limits
    pub mod api {
        pub const RATE_LIMIT_REQUESTS: u32 = 100;
        pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
    }
}
```

### 🔧 **Configuration Structure**

```rust
// config/app-config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

impl AppConfig {
    /// Load configuration from environment and files
    pub fn load() -> Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        config.try_deserialize()
    }

    /// Get configuration for specific environment
    pub fn load_for_env(env: &str) -> Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!("config/{}", env)).required(false))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        config.try_deserialize()
    }
}
```

### 🌍 **Environment-Specific Files**

```toml
# config/development.toml
[server]
host = "127.0.0.1"
port = 3000
workers = 1

[database]
url = "sqlite:dev.db"
max_connections = 5

[logging]
level = "debug"
```

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "${DATABASE_URL}"
max_connections = 20

[logging]
level = "info"
```

---

## Testing Strategies

### 🧪 **Test Organization by SRP**

```
libs/
└── geometry/
    ├── src/
    │   ├── sphere.rs
    │   └── cube.rs
    └── tests/
        ├── sphere-tests.rs      # Sphere-specific tests
        ├── cube-tests.rs         # Cube-specific tests
        ├── geometry-integration.rs  # Cross-primitive tests
        └── performance-tests.rs     # Performance benchmarks
```

### 📋 **Test Structure Templates**

```rust
// libs/geometry/src/tests/sphere-tests.rs
use proptest::prelude::*;
use insta::assert_snapshot;
use crate::sphere::{Sphere, SphereBuilder};

#[cfg(test)]
mod sphere_creation_tests {
    use super::*;

    #[test]
    fn test_sphere_with_positive_radius() {
        let sphere = Sphere::new(5.0);
        assert_eq!(sphere.radius(), 5.0);
        assert!(sphere.is_valid());
    }

    #[test]
    fn test_sphere_rejects_negative_radius() {
        let result = Sphere::new(-1.0);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod sphere_calculation_tests {
    use super::*;

    proptest! {
        #[test]
        fn test_sphere_volume_increases_with_radius(radius in 0.1f64..100.0) {
            let sphere = Sphere::new(radius);
            let volume = sphere.volume();

            prop_assert!(volume > 0.0);
            // Volume should be proportional to radius³
            let normalized_volume = volume / radius.powi(3);
            prop_assert!(normalized_volume > 4.0 && normalized_volume < 5.0);
        }
    }

    #[test]
    fn test_surface_area_calculation() {
        let sphere = Sphere::new(2.0);
        let surface_area = sphere.surface_area();

        // Surface area = 4πr² ≈ 50.265 for r=2
        assert!((surface_area - 50.265).abs() < 0.001);
    }
}

#[cfg(test)]
mod sphere_builder_tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let sphere = SphereBuilder::new()
            .with_radius(3.0)
            .with_center([1.0, 2.0, 3.0])
            .build();

        assert_eq!(sphere.radius(), 3.0);
        assert_eq!(sphere.center(), [1.0, 2.0, 3.0]);
    }
}
```

### 🚀 **Command-Line Testing**

```rust
// tests/cli-integration-tests.rs
use std::process::Command;
use tempfile::TempDir;

struct CommandOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

fn run_cli_command(args: &[&str]) -> CommandOutput {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "cli-tool", "--"])
        .args(args)
        .output()
        .expect("Failed to execute command");

    CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    }
}

#[test]
fn test_cli_processes_valid_input() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.json");

    // Create test input file
    std::fs::write(&input_file, r#"{"name": "test", "value": 42}"#).unwrap();

    let output = run_cli_command(&["process", input_file.to_str().unwrap()]);

    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("Processing complete"));
    assert!(output.stderr.is_empty());
}

#[test]
fn test_cli_shows_usage_for_invalid_input() {
    let output = run_cli_command(&["process", "nonexistent.json"]);

    assert_ne!(output.exit_code, 0);
    assert!(output.stderr.contains("File not found"));
}
```

---

## Performance Guidelines

### ⚡ **Algorithm Optimization**

```rust
// Before: O(n²) nested loop
fn find_pairs_slow(data: &[i32]) -> Vec<(i32, i32)> {
    let mut pairs = Vec::new();
    for i in 0..data.len() {
        for j in (i + 1)..data.len() {
            if data[i] + data[j] == 10 {
                pairs.push((data[i], data[j]));
            }
        }
    }
    pairs
}

// After: O(n) with hash map
use std::collections::HashMap;

fn find_pairs_optimized(data: &[i32]) -> Vec<(i32, i32)> {
    let mut seen: HashMap<i32, i32> = HashMap::new();
    let mut pairs = Vec::new();

    for &value in data {
        let complement = 10 - value;
        if let Some(&partner) = seen.get(&complement) {
            pairs.push((partner, value));
        }
        seen.insert(value, partner);
    }

    pairs
}
```

### 📊 **Memory Management**

```rust
// Prefer stack allocation for small, fixed-size data
#[inline]
fn point_distance(p1: [f64; 2], p2: [f64; 2]) -> f64 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    (dx * dx + dy * dy).sqrt()
}

// Use Vec::with_capacity for known sizes
fn process_data(input: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(input.len());
    for &item in input {
        result.push(item * 2);
    }
    result
}

// Avoid unnecessary allocations
fn transform_in_place(data: &mut [f32]) {
    for value in data.iter_mut() {
        *value = value.sqrt();
    }
}
```

### 🔍 **Profiling Integration**

```rust
// libs/utils/src/performance.rs
use std::time::Instant;

pub struct PerformanceTimer {
    name: String,
    start: Instant,
}

impl PerformanceTimer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        eprintln!("⏱️  {}: {:?}", self.name, elapsed);
    }
}

// Usage
fn expensive_operation() {
    let _timer = PerformanceTimer::new("expensive_operation");

    // ... operation code ...
}

// Benchmark testing
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_algorithm_improvement() {
        let data: Vec<i32> = (1..=10000).collect();

        let start = Instant::now();
        let _result1 = find_pairs_slow(&data);
        let slow_time = start.elapsed();

        let start = Instant::now();
        let _result2 = find_pairs_optimized(&data);
        let fast_time = start.elapsed();

        println!("Slow: {:?}, Fast: {:?}", slow_time, fast_time);
        assert!(fast_time < slow_time);
    }
}
```

---

## Common Pitfalls

### 🚫 **File Organization Pitfalls**

```rust
// ❌ Don't create monolithic files
// lib.rs (800+ lines with mixed concerns)

// ✅ Do split by responsibility
// lib.rs
pub mod auth;
pub mod users;
pub mod posts;

// auth/mod.rs
pub mod login;
pub mod registration;
pub mod session;

// auth/login.rs (single responsibility, <500 lines)
pub struct LoginService { /* ... */ }
```

### 🚫 **Configuration Pitfalls**

```rust
// ❌ Don't scatter configuration
// Multiple files with hardcoded values

// ✅ Do centralize configuration
// config/constants.rs - all constants in one place
// config/app-config.rs - structured configuration
```

### 🚫 **Testing Pitfalls**

```rust
// ❌ Don't test implementation details
#[test]
fn test_internal_function_called() {
    // Fragile, tests implementation
}

// ✅ Do test behavior
#[test]
fn test_input_produces_expected_output() {
    // Tests actual behavior
}
```

### 🚫 **Performance Pitfalls**

```rust
// ❌ Don't allocate unnecessarily
fn process_strings_bad(strings: &[String]) -> String {
    let mut result = String::new();
    for s in strings {
        result.push_str(&format!("{} ", s)); // allocates each iteration
    }
    result
}

// ✅ Do optimize allocations
fn process_strings_good(strings: &[String]) -> String {
    let capacity: usize = strings.iter().map(|s| s.len() + 1).sum();
    let mut result = String::with_capacity(capacity);
    for s in strings {
        result.push_str(s);
        result.push(' ');
    }
    result
}
```

---

## Development Workflow

### 🔄 **Incremental Development Process**

1. **Start Small**: Create minimal working version
2. **Add Tests First**: Write failing test, then implement
3. **Iterate**: Add small increments, test each step
4. **Refactor**: Improve structure without changing behavior
5. **Review**: Regular code reviews for SRP compliance

### 📝 **Git Workflow**

```bash
# Feature branch for small, focused changes
git checkout -b feature/user-authentication

# Commit small, logical changes
git add libs/auth/src/login.rs
git commit -m "feat: implement basic login functionality"

# Keep commits focused and descriptive
git add libs/auth/src/tests/login-tests.rs
git commit -m "test: add comprehensive login tests"
```

### 🔍 **Code Review Checklist**

- [ ] File under 500 lines
- [ ] Single responsibility clearly defined
- [ ] Tests cover behavior, not implementation
- [ ] Configuration externalized
- [ ] Performance considerations addressed
- [ ] Documentation updated
- [ ] No unused imports or dead code

### 📊 **Quality Metrics**

```bash
# Run quality checks
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo audit
cargo deny check
```

---

*This guide follows SOLID principles, emphasizes TDD without mocks, and maintains clean, modular code organization. Update this document as the project evolves to keep practices current and relevant.*