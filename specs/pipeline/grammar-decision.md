# Tree-sitter Grammar Variant Decision

**Date**: 2025-11-18  
**Status**: Confirmed  
**Decision**: Keep Current Local Grammar

## Executive Summary

After evaluating the existing local grammar against upstream alternatives, we will **keep the current local grammar** in `libs/openscad-parser/grammar.js`. The local grammar provides comprehensive OpenSCAD language support and integrates well with our Rust/WASM architecture.

## Grammar Evaluation

### Current Local Grammar Assessment

**Strengths:**
- ✅ **Comprehensive Coverage**: Supports all OpenSCAD constructs including primitives, transformations, booleans, control flow, and advanced features
- ✅ **Well-Structured**: Clean, modular grammar with helper functions for maintainability
- ✅ **Test Coverage**: Extensive test corpus with 114+ test cases across basic, intermediate, and advanced categories
- ✅ **Rust Integration**: Already integrated with Rust bindings under `bindings/rust/`
- ✅ **Performance**: Optimized for incremental parsing and error recovery

**Language Features Covered:**
```javascript
// 3D Primitives
cube(), sphere(), cylinder(), polyhedron()

// 2D Shapes  
circle(), square(), polygon(), text()

// Transformations
translate(), rotate(), scale(), mirror(), color(), resize()

// Boolean Operations
union(), difference(), intersection(), minkowski(), hull()

// Control Flow
if/else, for loops, let expressions, list comprehensions

// Advanced Features
modules, functions, includes, assertions, special variables ($fn, $fa, $fs)
```

### Upstream Grammar Comparison

**@holistic-stack/tree-sitter-openscad:**
- ✅ Claims 100% test coverage (114/114 tests)
- ✅ Production-ready with performance optimizations
- ✅ TypeScript support and comprehensive examples
- ⚠️ **Dependency Risk**: External dependency with potential breaking changes
- ⚠️ **Integration Overhead**: Would require reworking existing Rust bindings
- ⚠️ **Bundle Size**: Additional external dependency increases WASM size

**bollian/tree-sitter-openscad:**
- ✅ Basic OpenSCAD parsing capabilities
- ⚠️ **Limited Features**: Less comprehensive than local grammar
- ⚠️ **Maintenance Risk**: Smaller community, less active development

**nymann/tree-sitter-openscad:**
- ⚠️ **Minimal Documentation**: Limited information available
- ⚠️ **Unknown Coverage**: Unclear test coverage and feature completeness

## Decision Rationale

### Keep Local Grammar (Recommended)

**Advantages:**
1. **Full Control**: Complete ownership of grammar evolution and bug fixes
2. **Zero Dependencies**: No external dependencies for WASM bundle
3. **Tailored Integration**: Optimized for our specific Rust/WASM pipeline
4. **Proven Stability**: Already working with comprehensive test suite
5. **Performance**: Direct control over parsing performance optimizations

**Maintenance Strategy:**
- Regular grammar audits for OpenSCAD language updates
- Performance profiling and optimization
- Continuous test coverage expansion
- Documentation updates with examples

### Implementation Plan

#### Grammar Maintenance
1. **Quarterly Reviews**: Evaluate OpenSCAD language changes
2. **Performance Monitoring**: Track parsing speed and memory usage
3. **Test Expansion**: Add new test cases as OpenSCAD evolves
4. **Documentation**: Keep grammar documentation current

#### Integration Improvements
1. **Rust Bindings**: Enhance existing bindings for better performance
2. **Error Recovery**: Improve error messages and recovery strategies
3. **Incremental Parsing**: Optimize for real-time editor integration
4. **WASM Optimization**: Minimize grammar size for faster loading

### Risk Mitigation

**Grammar Stagnation Risk:**
- ✅ **Active Monitoring**: Track OpenSCAD releases for language changes
- ✅ **Community Feedback**: Accept contributions for new language features
- ✅ **Fallback Plan**: Can migrate to upstream grammar if needed

**Performance Degradation Risk:**
- ✅ **Benchmarking**: Regular performance testing with real-world files
- ✅ **Profiling**: Use `tree-sitter parse` profiling tools
- ✅ **Optimization**: Apply tree-sitter best practices for performance

### Success Metrics

- **Parse Speed**: Maintain >5MB/s parsing speed for typical files
- **Error Rate**: <0.1% parsing failures on valid OpenSCAD code
- **Bundle Size**: Grammar contributes <100KB to WASM bundle
- **Test Coverage**: Maintain >95% language feature coverage

## Conclusion

The local grammar provides the best balance of feature completeness, performance, and integration simplicity for our Rust OpenSCAD pipeline. By maintaining control over the grammar, we ensure optimal performance and compatibility while avoiding external dependency risks.

**Next Steps:**
1. ✅ Document grammar maintenance procedures
2. ✅ Set up performance monitoring
3. ✅ Plan quarterly grammar reviews
4. ✅ Enhance error recovery mechanisms