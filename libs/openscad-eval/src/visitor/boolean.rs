//! # Boolean Operation Evaluators
//!
//! Evaluators for CSG boolean operations.
//!
//! ## Operations
//!
//! - `union()` - Combine geometries
//! - `difference()` - Subtract geometries
//! - `intersection()` - Keep only overlapping region
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = eval_union(&mut ctx, &children)?;
//! ```

use crate::error::EvalError;
use crate::geometry::GeometryNode;
use openscad_ast::Statement;

use super::context::{EvalContext, evaluate_statement};

// =============================================================================
// BOOLEAN OPERATIONS
// =============================================================================

/// Evaluate union() call.
///
/// Combines all child geometries into a single geometry.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `children`: Child statements to combine
///
/// ## Example
///
/// ```text
/// union() {
///     cube(10);
///     translate([5, 0, 0]) cube(10);
/// }
/// ```
pub fn eval_union(
    ctx: &mut EvalContext,
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let child_nodes = flatten_children(ctx, children)?;

    match child_nodes.len() {
        0 => Ok(GeometryNode::Empty),
        1 => Ok(child_nodes.into_iter().next().unwrap()),
        _ => Ok(GeometryNode::Union { children: child_nodes }),
    }
}

/// Evaluate difference() call.
///
/// Subtracts all subsequent children from the first child.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `children`: Child statements (first is base, rest are subtracted)
///
/// ## Example
///
/// ```text
/// difference() {
///     cube(10);
///     sphere(6);  // Subtracted from cube
/// }
/// ```
pub fn eval_difference(
    ctx: &mut EvalContext,
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let child_nodes = flatten_children(ctx, children)?;

    if child_nodes.is_empty() {
        Ok(GeometryNode::Empty)
    } else {
        Ok(GeometryNode::Difference { children: child_nodes })
    }
}

/// Evaluate intersection() call.
///
/// Keeps only the overlapping region of all children.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `children`: Child statements to intersect
///
/// ## Example
///
/// ```text
/// intersection() {
///     cube(10, center=true);
///     sphere(6);  // Keep only overlap
/// }
/// ```
pub fn eval_intersection(
    ctx: &mut EvalContext,
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let child_nodes = flatten_children(ctx, children)?;

    if child_nodes.is_empty() {
        Ok(GeometryNode::Empty)
    } else {
        Ok(GeometryNode::Intersection { children: child_nodes })
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Flatten children, extracting Block contents.
///
/// When a module call has a block body like `union() { ... }`,
/// the children contain a single Block statement that needs to be expanded.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `children`: Child statements to flatten
///
/// ## Returns
///
/// Flattened list of geometry nodes
fn flatten_children(
    ctx: &mut EvalContext,
    children: &[Statement],
) -> Result<Vec<GeometryNode>, EvalError> {
    let mut result = Vec::new();

    for stmt in children {
        match stmt {
            // Extract statements from blocks
            Statement::Block { statements, .. } => {
                for inner in statements {
                    if let Some(node) = evaluate_statement(ctx, inner)? {
                        if !node.is_empty() {
                            result.push(node);
                        }
                    }
                }
            }
            // Regular statements
            _ => {
                if let Some(node) = evaluate_statement(ctx, stmt)? {
                    if !node.is_empty() {
                        result.push(node);
                    }
                }
            }
        }
    }

    Ok(result)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> EvalContext {
        EvalContext::new()
    }

    #[test]
    fn test_eval_union_empty() {
        let mut ctx = ctx();
        let node = eval_union(&mut ctx, &[]).unwrap();
        assert!(matches!(node, GeometryNode::Empty));
    }

    #[test]
    fn test_eval_difference_empty() {
        let mut ctx = ctx();
        let node = eval_difference(&mut ctx, &[]).unwrap();
        assert!(matches!(node, GeometryNode::Empty));
    }

    #[test]
    fn test_eval_intersection_empty() {
        let mut ctx = ctx();
        let node = eval_intersection(&mut ctx, &[]).unwrap();
        assert!(matches!(node, GeometryNode::Empty));
    }
}
