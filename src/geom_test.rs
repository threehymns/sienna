use crate::geom::{BezierCubic, BezierQuad, Point};
use gpui::Point as GpuiPoint;

fn assert_point_eq(p1: Point, p2: Point) {
    let diff_x = (p1.x - p2.x).abs();
    let diff_y = (p1.y - p2.y).abs();
    assert!(
        diff_x < 1e-5 && diff_y < 1e-5,
        "Expected {:?}, got {:?}",
        p2,
        p1
    );
}

#[test]
fn test_point_conversion() {
    let gpui_p = GpuiPoint { x: 10.0, y: 20.0 };
    let p: Point = gpui_p.into();
    assert_eq!(p.x, 10.0);
    assert_eq!(p.y, 20.0);

    let gpui_p2: GpuiPoint<f32> = p.into();
    assert_eq!(gpui_p2.x, 10.0);
    assert_eq!(gpui_p2.y, 20.0);
}

#[test]
fn test_bezier_quad_evaluate() {
    let quad = BezierQuad::new(
        Point::new(0.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(20.0, 0.0),
    );

    assert_point_eq(quad.evaluate(0.0), Point::new(0.0, 0.0));
    assert_point_eq(quad.evaluate(0.5), Point::new(10.0, 5.0));
    assert_point_eq(quad.evaluate(1.0), Point::new(20.0, 0.0));
}

#[test]
fn test_bezier_quad_split() {
    let quad = BezierQuad::new(
        Point::new(0.0, 0.0),
        Point::new(10.0, 10.0),
        Point::new(20.0, 0.0),
    );

    let (left, right) = quad.split(0.5);

    assert_point_eq(left.start, Point::new(0.0, 0.0));
    assert_point_eq(left.control, Point::new(5.0, 5.0));
    assert_point_eq(left.end, Point::new(10.0, 5.0));

    assert_point_eq(right.start, Point::new(10.0, 5.0));
    assert_point_eq(right.control, Point::new(15.0, 5.0));
    assert_point_eq(right.end, Point::new(20.0, 0.0));
}

#[test]
fn test_bezier_cubic_evaluate() {
    let cubic = BezierCubic::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 0.0),
    );

    assert_point_eq(cubic.evaluate(0.0), Point::new(0.0, 0.0));
    // At t=0.5:
    // a = (0, 5)
    // b = (5, 10)
    // c = (10, 5)
    // d = (2.5, 7.5)
    // e = (7.5, 7.5)
    // p = (5, 7.5)
    assert_point_eq(cubic.evaluate(0.5), Point::new(5.0, 7.5));
    assert_point_eq(cubic.evaluate(1.0), Point::new(10.0, 0.0));
}

#[test]
fn test_bezier_cubic_split() {
    let cubic = BezierCubic::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 10.0),
        Point::new(10.0, 10.0),
        Point::new(10.0, 0.0),
    );

    let (left, right) = cubic.split(0.5);

    assert_point_eq(left.start, Point::new(0.0, 0.0));
    assert_point_eq(left.control1, Point::new(0.0, 5.0));
    assert_point_eq(left.control2, Point::new(2.5, 7.5));
    assert_point_eq(left.end, Point::new(5.0, 7.5));

    assert_point_eq(right.start, Point::new(5.0, 7.5));
    assert_point_eq(right.control1, Point::new(7.5, 7.5));
    assert_point_eq(right.control2, Point::new(10.0, 5.0));
    assert_point_eq(right.end, Point::new(10.0, 0.0));
}
