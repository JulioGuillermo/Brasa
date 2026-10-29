//! A damped spring: a value that glides to its target and can be retargeted mid-flight without a jump.

const STIFFNESS: f64 = 320.0;
const DAMPING_RATIO: f64 = 0.78;
const REST_DISTANCE: f64 = 0.05;
const REST_SPEED: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub value: f64,
    pub target: f64,
    speed: f64,
}

impl Spring {
    pub fn new(value: f64) -> Self {
        Spring { value, target: value, speed: 0.0 }
    }

    pub fn step(&mut self, seconds: f64) {
        let damping = 2.0 * DAMPING_RATIO * STIFFNESS.sqrt();
        let pull = STIFFNESS * (self.target - self.value);
        self.speed += (pull - damping * self.speed) * seconds;
        self.value += self.speed * seconds;
        if self.at_rest() {
            self.value = self.target;
            self.speed = 0.0;
        }
    }

    pub fn at_rest(&self) -> bool {
        (self.target - self.value).abs() < REST_DISTANCE && self.speed.abs() < REST_SPEED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settles_on_its_target() {
        let mut spring = Spring::new(0.0);
        spring.target = 100.0;
        for _ in 0..240 {
            spring.step(1.0 / 60.0);
        }
        assert!(spring.at_rest());
        assert_eq!(spring.value, 100.0);
    }
}
