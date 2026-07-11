use gpui::Point as GpuiPoint;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl From<GpuiPoint<f32>> for Point {
    fn from(p: GpuiPoint<f32>) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<Point> for GpuiPoint<f32> {
    fn from(val: Point) -> Self {
        GpuiPoint { x: val.x, y: val.y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BezierQuad {
    pub start: Point,
    pub control: Point,
    pub end: Point,
}

impl BezierQuad {
    pub fn new(start: Point, control: Point, end: Point) -> Self {
        Self {
            start,
            control,
            end,
        }
    }

    pub fn evaluate(&self, t: f32) -> Point {
        self.split(t).0.end
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
        let a = self.start.lerp(self.control, t);
        let b = self.control.lerp(self.end, t);
        let p = a.lerp(b, t);

        let left = BezierQuad::new(self.start, a, p);
        let right = BezierQuad::new(p, b, self.end);

        (left, right)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BezierCubic {
    pub start: Point,
    pub control1: Point,
    pub control2: Point,
    pub end: Point,
}

impl BezierCubic {
    pub fn new(start: Point, control1: Point, control2: Point, end: Point) -> Self {
        Self {
            start,
            control1,
            control2,
            end,
        }
    }

    pub fn evaluate(&self, t: f32) -> Point {
        self.split(t).0.end
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
        let a = self.start.lerp(self.control1, t);
        let b = self.control1.lerp(self.control2, t);
        let c = self.control2.lerp(self.end, t);

        let d = a.lerp(b, t);
        let e = b.lerp(c, t);

        let p = d.lerp(e, t);

        let left = BezierCubic::new(self.start, a, d, p);
        let right = BezierCubic::new(p, e, c, self.end);

        (left, right)
    }
}
