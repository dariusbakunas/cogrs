use std::ops::{BitAnd, BitOr};

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum FailedState {
    None = 0,
    Setup = 1,
    Tasks = 1 << 2,
    Rescue = 1 << 3,
    Always = 1 << 4,
    Handlers = 1 << 5,
}

impl BitOr for FailedState {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u8) | (rhs as u8)
    }
}

impl BitAnd for FailedState {
    type Output = u8;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u8) & (rhs as u8)
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct FailedStates(u8);

impl FailedStates {
    /// Create a new `FailedStates` with `FailedState::None` flag set
    pub fn new() -> Self {
        FailedStates(FailedState::None as u8)
    }

    /// Add a state to the flags
    pub fn add(&mut self, state: FailedState) {
        self.0 |= state as u8;
    }

    /// Remove a state from the flags
    pub fn remove(&mut self, state: FailedState) {
        self.0 &= !(state as u8);
    }

    /// Check if the flags contain a specific state
    pub fn contains(&self, state: FailedState) -> bool {
        (self.0 & (state as u8)) != 0
    }
}

impl BitOr for FailedStates {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        FailedStates(self.0 | rhs.0)
    }
}

impl BitOr<FailedState> for FailedStates {
    type Output = Self;

    fn bitor(self, rhs: FailedState) -> Self::Output {
        FailedStates(self.0 | rhs as u8)
    }
}

impl BitAnd<FailedState> for FailedStates {
    type Output = Self;

    fn bitand(self, rhs: FailedState) -> Self::Output {
        FailedStates(self.0 & rhs as u8)
    }
}

impl PartialEq<u8> for FailedStates {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl PartialEq for FailedStates {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<FailedState> for FailedStates {
    fn eq(&self, other: &FailedState) -> bool {
        self.0 == *other as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let states = FailedStates::new();
        assert_eq!(states, FailedState::None);
    }

    #[test]
    fn test_add_state() {
        let mut states = FailedStates::new();
        states.add(FailedState::Setup);
        assert!(
            states.contains(FailedState::Setup),
            "Setup state should be added"
        );
        assert_eq!(
            states,
            FailedState::Setup,
            "FailedStates should equal the Setup FailedState"
        );
    }

    #[test]
    fn test_remove_state() {
        let mut states = FailedStates::new();
        states.add(FailedState::Tasks);
        assert!(
            states.contains(FailedState::Tasks),
            "Tasks state should be added"
        );

        states.remove(FailedState::Tasks);
        assert!(
            !states.contains(FailedState::Tasks),
            "Tasks state should be removed"
        );
    }

    #[test]
    fn test_combined_states() {
        let mut states = FailedStates::new();
        states.add(FailedState::Setup);
        states.add(FailedState::Tasks);

        assert!(
            states.contains(FailedState::Setup),
            "Setup state should be present"
        );
        assert!(
            states.contains(FailedState::Tasks),
            "Tasks state should be present"
        );

        states.remove(FailedState::Setup);
        assert!(
            !states.contains(FailedState::Setup),
            "Setup state should be removed"
        );
        assert!(
            states.contains(FailedState::Tasks),
            "Tasks state should still be present"
        );
    }

    #[test]
    fn test_is_empty() {
        let states = FailedStates::new();
        assert_eq!(
            states,
            FailedState::None,
            "New FailedStates should be empty"
        );

        let mut states_with_task = states;
        states_with_task.add(FailedState::Tasks);
        assert_eq!(
            states_with_task,
            FailedState::Tasks,
            "FailedStates should be equal to Tasks"
        );
    }

    #[test]
    fn test_bitwise_or() {
        let mut state1 = FailedStates::new();
        state1.add(FailedState::Setup);

        let mut state2 = FailedStates::new();
        state2.add(FailedState::Tasks);

        let combined = state1 | state2;

        assert!(
            combined.contains(FailedState::Setup),
            "Setup state should be in combined FailedStates"
        );
        assert!(
            combined.contains(FailedState::Tasks),
            "Tasks state should be in combined FailedStates"
        );
    }

    #[test]
    fn test_bitwise_or_with_failedstate() {
        let mut states = FailedStates::new();
        states.add(FailedState::Setup);

        let combined = states | FailedState::Tasks;

        assert!(
            combined.contains(FailedState::Setup),
            "Setup state should be in combined FailedStates"
        );
        assert!(
            combined.contains(FailedState::Tasks),
            "Tasks state should be in combined FailedStates"
        );
    }

    #[test]
    fn test_bitwise_and_with_failedstate() {
        let mut states = FailedStates::new();
        states.add(FailedState::Setup);
        states.add(FailedState::Tasks);

        let result = states & FailedState::Setup;

        assert!(
            result.contains(FailedState::Setup),
            "FailedStates should contain Setup state after bitwise AND"
        );
        assert!(
            !result.contains(FailedState::Tasks),
            "FailedStates should not contain Tasks state after bitwise AND with Setup"
        );
    }

    #[test]
    fn test_partial_eq_with_u8() {
        let mut states = FailedStates::new();
        states.add(FailedState::Setup);

        assert_eq!(
            states,
            FailedState::Setup as u8,
            "FailedStates should equal the corresponding u8 representation"
        );
    }

    #[test]
    fn test_partial_eq_with_failedstates() {
        let mut states1 = FailedStates::new();
        states1.add(FailedState::Setup);

        let mut states2 = FailedStates::new();
        states2.add(FailedState::Setup);

        assert_eq!(
            states1, states2,
            "Two FailedStates with the same states should be equal"
        );

        states2.add(FailedState::Tasks);
        assert_ne!(
            states1, states2,
            "Two FailedStates with different states should not be equal"
        );
    }
}
