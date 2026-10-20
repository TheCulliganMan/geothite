//! Continuous controller position relative to the authoritative gameplay cell.
//! Cell crossings still go through the production world controller.
use super::map::Direction;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FreeMovement {
    pub offset: [f32; 2],
}

impl FreeMovement {
    /// Integrate one fixed gameplay tick. The caller probes each neighboring
    /// cell with the ordinary collision rules before allowing an approach.
    pub fn advance(&mut self, mut stick: [f32; 2], speed: f32, open: [bool; 4]) -> Option<Direction> {
        let length = stick[0].hypot(stick[1]);
        if !length.is_finite() || !speed.is_finite() { return None; }
        if length > 1.0 { for v in &mut stick { *v /= length; } }
        for axis in 0..2 {
            self.offset[axis] += stick[axis] * speed.clamp(0.0, 0.2);
            // A small player capsule leaves clearance from solid tile faces.
            let negative = if open[axis * 2] { 0.7 } else { 0.22 };
            let positive = if open[axis * 2 + 1] { 0.7 } else { 0.22 };
            self.offset[axis] = self.offset[axis].clamp(-negative, positive);
        }
        let [x, y] = self.offset;
        if x.abs() >= 0.5 && x.abs() >= y.abs() {
            Some(if x < 0.0 { Direction::Left } else { Direction::Right })
        } else if y.abs() >= 0.5 {
            Some(if y < 0.0 { Direction::Up } else { Direction::Down })
        } else { None }
    }

    pub fn crossed(&mut self, direction: Direction) {
        let (axis, delta) = match direction {
            Direction::Left => (0, -1.0), Direction::Right => (0, 1.0),
            Direction::Up => (1, -1.0), Direction::Down => (1, 1.0),
        };
        self.offset[axis] -= delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stops_between_cells_and_reverses_without_snapping() {
        let mut p = FreeMovement::default();
        assert_eq!(p.advance([0.6, 0.0], 0.1, [true;4]), None);
        assert_eq!(p.advance([0.0, 0.0], 0.1, [true;4]), None);
        assert!((p.offset[0] - 0.06).abs() < 1e-6);
        p.advance([-0.3, 0.0], 0.1, [true;4]);
        assert!((p.offset[0] - 0.03).abs() < 1e-6);
    }
    #[test]
    fn diagonal_speed_is_normalized_and_walls_slide() {
        let mut p = FreeMovement::default();
        p.advance([1.0, 1.0], 0.1, [true;4]);
        assert!((p.offset[0].hypot(p.offset[1]) - 0.1).abs() < 1e-6);
        for _ in 0..20 { p.advance([1.0, 1.0], 0.1, [true,false,true,true]); }
        assert_eq!(p.offset[0], 0.22);
        assert!(p.offset[1] >= 0.5);
    }
    #[test]
    fn cell_commit_preserves_continuous_position() {
        let mut p = FreeMovement { offset: [0.49, 0.15] };
        assert_eq!(p.advance([1.0,0.0],0.05,[true;4]),Some(Direction::Right));
        let before = p.offset[0]; p.crossed(Direction::Right);
        assert!((before - (1.0 + p.offset[0])).abs() < 1e-6);
        assert_eq!(p.offset[1],0.15);
    }
}
