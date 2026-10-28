"""A damped spring: a value that glides to its target and can be retargeted mid-flight without a jump."""

STIFFNESS = 320.0
DAMPING_RATIO = 0.78
REST_DISTANCE = 0.05
REST_SPEED = 0.5


class Spring:
    def __init__(self, value: float, stiffness: float = STIFFNESS, damping_ratio: float = DAMPING_RATIO):
        self.value = value
        self.target = value
        self.speed = 0.0
        self.stiffness = stiffness
        self.damping = 2 * damping_ratio * stiffness ** 0.5

    def step(self, seconds: float):
        pull = self.stiffness * (self.target - self.value)
        self.speed += (pull - self.damping * self.speed) * seconds
        self.value += self.speed * seconds
        if self.at_rest():
            self.value = self.target
            self.speed = 0.0

    def at_rest(self) -> bool:
        return abs(self.target - self.value) < REST_DISTANCE and abs(self.speed) < REST_SPEED
