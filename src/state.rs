use crate::state_machine::Transition;
use bracket_lib::prelude::*;
use std::time::Duration;

pub type StateTransition<S, R> = Transition<S, R>;

/// Desired behavior for the next update, to be returned from an `update` call.
#[derive(Debug)]
pub enum TransitionUpdate {
    /// Run the next update immediately, without waiting for the next frame.
    Immediate,
    /// Wait a frame before the next update; this will likely draw the mode for a frame.
    Update,
    /// Wait for an input event before the next update; this will likely draw the mode before
    /// waiting.
    WaitForEvent,
}

pub trait State {
    type State: ?Sized;
    type StateResult;

    #[must_use = "it may trigger a state change"]
    fn update(
        &mut self,
        term: &mut BTerm,
        state: &Self::State,
        pop_result: &Option<Self::StateResult>,
        dt: Duration,
    ) -> (
        StateTransition<Self::State, Self::StateResult>,
        TransitionUpdate,
    )
    where
        Self::State: std::marker::Sized,
        Self::StateResult: std::marker::Sized;

    fn render(&self, term: &mut BTerm, state: &Self::State, active: bool);

    fn clear(&self, _state: &Self::State, _term: &mut BTerm) {
        BACKEND_INTERNAL
            .lock()
            .consoles
            .iter_mut()
            .for_each(|c| c.console.cls());
    }

    fn is_transparent(&self) -> bool {
        true
    }
}
