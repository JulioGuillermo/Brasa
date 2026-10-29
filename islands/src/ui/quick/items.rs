//! What the panel can hold: tiles that switch something, tiles that run something, and tiles that open a
//! menu of choices.

/// Lit while `state` (a shell test) succeeds; a click runs `on` or `off`.
#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    pub icon: String,
    pub name: String,
    pub on: String,
    pub off: String,
    pub state: String,
}

/// A click closes the panel and runs `command`.
#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub icon: String,
    pub name: String,
    pub command: String,
}

/// One entry of a menu, lit while `current` (a shell test) succeeds.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub label: String,
    pub command: String,
    pub current: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub icon: String,
    pub name: String,
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Switch(Switch),
    Action(Action),
    Menu(Menu),
}

impl Item {
    pub fn icon(&self) -> &str {
        match self {
            Item::Switch(switch) => &switch.icon,
            Item::Action(action) => &action.icon,
            Item::Menu(menu) => &menu.icon,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Item::Switch(switch) => &switch.name,
            Item::Action(action) => &action.name,
            Item::Menu(menu) => &menu.name,
        }
    }
}

pub fn switch(icon: &str, name: &str, on: &str, off: &str, state: &str) -> Item {
    Item::Switch(Switch { icon: icon.into(), name: name.into(), on: on.into(), off: off.into(), state: state.into() })
}

pub fn action(icon: &str, name: &str, command: &str) -> Item {
    Item::Action(Action { icon: icon.into(), name: name.into(), command: command.into() })
}

pub fn choice(label: &str, command: &str, current: &str) -> Choice {
    Choice { label: label.into(), command: command.into(), current: current.into() }
}

pub fn menu(icon: &str, name: &str, choices: Vec<Choice>) -> Item {
    Item::Menu(Menu { icon: icon.into(), name: name.into(), choices })
}
