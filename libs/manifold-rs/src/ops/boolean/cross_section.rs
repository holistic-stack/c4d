use crate::core::cross_section::CrossSection;
use crate::BooleanOp;
use cavalier_contours::polyline::*;
use glam::DVec2;

#[allow(dead_code)]
pub fn boolean_2d(
    cs_a: &CrossSection,
    cs_b: &CrossSection,
    op: BooleanOp
) -> Result<CrossSection, String> {

    let plines_a = to_plines(cs_a);
    let plines_b = to_plines(cs_b);

    if plines_a.is_empty() { return Ok(cs_b.clone()); }
    if plines_b.is_empty() { return Ok(cs_a.clone()); }

    let pline_a = &plines_a[0];
    let pline_b = &plines_b[0];

    let cav_op = match op {
        BooleanOp::Union => cavalier_contours::polyline::BooleanOp::Or,
        BooleanOp::Difference => cavalier_contours::polyline::BooleanOp::Not,
        BooleanOp::Intersection => cavalier_contours::polyline::BooleanOp::And,
    };

    // Boolean returns BooleanResult which has pos_plines and neg_plines
    let result = pline_a.boolean(pline_b, cav_op);

    let mut out_contours = Vec::new();

    for wrapper in result.pos_plines {
         out_contours.push(extract_points_from_wrapper(&wrapper));
    }

    for wrapper in result.neg_plines {
         out_contours.push(extract_points_from_wrapper(&wrapper));
    }

    Ok(CrossSection::from_contours(out_contours))
}

fn to_plines(cs: &CrossSection) -> Vec<Polyline> {
    let mut plines = Vec::new();
    for contour in &cs.contours {
        let mut pline = Polyline::new();
        for p in contour {
            pline.add(p.x, p.y, 0.0);
        }
        pline.set_is_closed(true);
        plines.push(pline);
    }
    plines
}

fn extract_points_from_wrapper(wrapper: &BooleanResultPline<Polyline>) -> Vec<DVec2> {
    let mut pts = Vec::new();
    // Use the exposed .pline field
    let count = wrapper.pline.vertex_count();
    for i in 0..count {
        // We use indexing or get_vertex if PlineSource is used
        // But since we access pline directly, we can use its methods.
        // wrapper.pline is a Polyline.
        // Polyline has vertex_data: Vec<PlineVertex>.
        let v = wrapper.pline.vertex_data[i];
        pts.push(DVec2::new(v.x, v.y));
    }
    pts
}
