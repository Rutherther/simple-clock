use alloc::vec::Vec;

pub struct Point<Position, Value> {
    position: Position,
    value: Value,
}

impl<Position, Value> Point<Position, Value> {
    pub fn new(position: Position, value: Value) -> Self {
        Self { position, value }
    }
}

impl<Position: Copy, Value: Copy> Point<Position, Value> {
    pub fn deconstruct(&self) -> (Position, Value) {
        (self.position, self.value)
    }
}

pub struct LinearInterpolation<Position, Value> {
    points: Vec<Point<Position, Value>>,
}

impl<Position: Ord + PartialOrd + Copy, Value: Copy> LinearInterpolation<Position, Value> {
    pub fn closest_position_indices(&self, position: Position) -> Option<(usize, usize)> {
        let mut closest_lower_index = 0;
        let mut closest_upper_index = self.points.len() - 1;
        let p = &self.points;
        for i in 1..self.points.len() {
            let curr_position = p[i].position;
            if curr_position > p[closest_lower_index].position && curr_position <= position {
                closest_lower_index = i;
            }

            if curr_position < p[closest_upper_index].position && curr_position >= position {
                closest_upper_index = i;
            }
        }

        if p[closest_lower_index].position <= position
            && p[closest_upper_index].position >= position
        {
            Some((closest_lower_index, closest_upper_index))
        } else {
            None
        }
    }
}

impl LinearInterpolation<u16, u16> {
    pub fn new(points: Vec<Point<u16, u16>>) -> Self {
        Self { points }
    }

    pub fn interpolate(&self, position: u16) -> Option<u16> {
        let (lower, upper) = self.closest_position_indices(position)?;

        if lower == upper {
            return Some(self.points[lower].value);
        }

        let (lower, upper) = (&self.points[lower], &self.points[upper]);

        let diff = upper.position - lower.position;
        let value_diff = upper.value as i32 - lower.value as i32;
        let position_relative = (position - lower.position) as f32 / diff as f32;

        Some((lower.value as f32 + (value_diff as f32 * position_relative)) as u16)
    }
}
