"""What the panel can hold: tiles that switch something on or off, tiles that run something, and tiles
that open a menu of choices."""
from dataclasses import dataclass, field


@dataclass
class Switch:
    """Lit while `state` (a shell test) succeeds; a click runs `on` or `off`."""
    icon: str
    name: str
    on: str
    off: str
    state: str


@dataclass
class Action:
    """A click closes the panel and runs `command`."""
    icon: str
    name: str
    command: str


@dataclass
class Choice:
    """One entry of a menu, lit while `current` (a shell test) succeeds."""
    label: str
    command: str
    current: str


@dataclass
class Menu:
    icon: str
    name: str
    choices: list[Choice] = field(default_factory=list)
