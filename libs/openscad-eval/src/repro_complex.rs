
#[cfg(test)]
mod tests {
    use crate::{evaluate, GeometryNode, BooleanOperation};

    #[test]
    fn test_repro_complex_scad() {
        let source = r#"
            // CSG-modules.scad - Basic usage of modules, if, color, $fs/$fa 
            
            // Change this to false to remove the helper geometry 
            debug = true; 
            
            // Global resolution 
            $fs = 0.1;  // Don't generate smaller facets than 0.1 mm 
            $fa = 5;    // Don't generate larger angles than 5 degrees 
            
            // Main geometry 
            difference() { 
                intersection() { 
                    body(); 
                    intersector(); 
                } 
                holes(); 
            } 
            
            // Helpers 
            if (debug) helpers(); 
            
            // Core geometric primitives. 
            // These can be modified to create variations of the final object 
            
            module body() { 
                color("Blue") sphere(10); 
            } 
            
            module intersector() { 
                color("Red") cube(15, center=true); 
            } 
            
            module holeObject() { 
                color("Lime") cylinder(h=20, r=5, center=true); 
            } 
            
            // Various modules for visualizing intermediate components 
            
            module intersected() { 
                intersection() { 
                    body(); 
                    intersector(); 
                } 
            } 
            
            module holeA() rotate([0,90,0]) holeObject(); 
            module holeB() rotate([90,0,0]) holeObject(); 
            module holeC() holeObject(); 
            
            module holes() { 
                union() { 
                    holeA(); 
                    holeB(); 
                    holeC(); 
                } 
            } 
            
            module helpers() { 
                // Inner module since it's only needed inside helpers 
                module line() color("Black") cylinder(r=1, h=10, center=true); 
            
                scale(0.5) { 
                    translate([-30,0,-40]) { 
                        intersected(); 
                        translate([-15,0,-35]) body(); 
                        translate([15,0,-35]) intersector(); 
                        translate([-7.5,0,-17.5]) rotate([0,30,0]) line(); 
                        translate([7.5,0,-17.5]) rotate([0,-30,0]) line(); 
                    } 
                    translate([30,0,-40]) { 
                        holes(); 
                        translate([-10,0,-35]) holeA(); 
                        translate([10,0,-35]) holeB(); 
                        translate([30,0,-35]) holeC(); 
                        translate([5,0,-17.5]) rotate([0,-20,0]) line(); 
                        translate([-5,0,-17.5]) rotate([0,30,0]) line(); 
                        translate([15,0,-17.5]) rotate([0,-45,0]) line(); 
                    } 
                    translate([-20,0,-22.5]) rotate([0,45,0]) line(); 
                    translate([20,0,-22.5]) rotate([0,-45,0]) line(); 
                } 
            } 
            
            echo(version=version()); 
        "#;

        let result = evaluate(source);
        
        // Print error if any
        if let Err(e) = &result {
            println!("Evaluation failed: {:?}", e);
        }

        assert!(result.is_ok(), "Evaluation should succeed");
        let nodes = result.unwrap();
        
        // We expect:
        // 1. The main geometry (difference)
        // 2. The helpers (since debug=true)
        //    - helpers module calls scale(0.5) which contains 3 translates
        //    - The 3 translates contain various children
        
        // Let's count top-level nodes
        // One 'difference' node
        // One 'scale' node (from helpers)
        // echo is not a geometry node
        
        assert_eq!(nodes.len(), 2, "Should have 2 top level geometry nodes");
        
        // First node: difference
        match &nodes[0] {
            GeometryNode::Boolean { operation, children, .. } => {
                assert_eq!(*operation, BooleanOperation::Difference);
                assert_eq!(children.len(), 2);
            },
            _ => panic!("First node should be Difference"),
        }

        // Second node: helpers -> scale
        match &nodes[1] {
            GeometryNode::Transform { children, .. } => {
                // Scale matrix check?
                assert_eq!(children.len(), 4); // 4 translates inside helpers
            },
            _ => panic!("Second node should be Transform (scale) from helpers"),
        }
    }
}
