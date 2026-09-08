use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtracting_an_inner_rectangle_preserves_visible_area_without_overlap() {
        let window = Rect::new(-100, -50, 200, 100);
        let occluder = Rect::new(-20, -10, 40, 20);
        let remaining = window.subtract(&occluder);
        assert_eq!(remaining.iter().map(Rect::area).sum::<i32>(), window.area() - occluder.area());
        for (i, rect) in remaining.iter().enumerate() {
            assert!(window.contains(rect));
            assert!(!rect.intersects(&occluder));
            assert!(remaining[..i].iter().all(|other| !rect.intersects(other)));
        }
    }

    #[test]
    fn touching_edges_do_not_occlude_each_other() {
        let left = Rect::new(0, 0, 100, 100);
        let right = Rect::new(100, 0, 100, 100);
        assert!(!left.intersects(&right));
        assert_eq!(left.subtract(&right).iter().map(Rect::area).sum::<i32>(), left.area());
        assert!(!left.contains_point(100, 50));
        assert!(right.contains_point(100, 50));
    }
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.width >= other.x + other.width
            && self.y + self.height >= other.y + other.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x + self.width <= other.x
            || other.x + other.width <= self.x
            || self.y + self.height <= other.y
            || other.y + other.height <= self.y)
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x + self.width).min(other.x + other.width) - x;
        let height = (self.y + self.height).min(other.y + other.height) - y;

        Some(Rect::new(x, y, width, height))
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    pub fn subtract(&self, other: &Rect) -> Vec<Rect> {
        if !self.intersects(other) {
            return vec![self.clone()];
        }

        let intersection = self.intersection(other).unwrap();
        let mut result = Vec::new();

        // 上方矩形
        if self.y < intersection.y {
            result.push(Rect::new(
                self.x,
                self.y,
                self.width,
                intersection.y - self.y,
            ));
        }

        // 下方矩形
        if self.y + self.height > intersection.y + intersection.height {
            result.push(Rect::new(
                self.x,
                intersection.y + intersection.height,
                self.width,
                (self.y + self.height) - (intersection.y + intersection.height),
            ));
        }

        // 左侧矩形
        if self.x < intersection.x {
            result.push(Rect::new(
                self.x,
                intersection.y,
                intersection.x - self.x,
                intersection.height,
            ));
        }

        // 右侧矩形
        if self.x + self.width > intersection.x + intersection.width {
            result.push(Rect::new(
                intersection.x + intersection.width,
                intersection.y,
                (self.x + self.width) - (intersection.x + intersection.width),
                intersection.height,
            ));
        }

        result
    }
}
