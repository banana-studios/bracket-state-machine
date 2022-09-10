use crate::state_machine::DeltaTime;
use bracket_lib::prelude::*;

pub type StateTransition<S, R> = Option<Transition<S, R>>;

pub trait State {
    type State: ?Sized;
    type StateResult: ?Sized;

    #[must_use = "it may trigger a state change"]
    fn update(
        &mut self,
        term: &mut BTerm,
        state: &Self::State,
        pop_result: &Option<Self::StateResult>,
        dt: DeltaTime,
    ) -> (
        StateTransition<Self::State, Self::StateResult>,
        TransitionUpdate,
    )
    where
        Self::State: std::marker::Sized,
        Self::StateResult: std::marker::Sized;

    fn draw(&self, _term: &mut BTerm, _state: &Self::State, _active: bool) {}

    fn clear_consoles(&self, _state: &Self::State, _term: &mut BTerm) {
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

pub enum Transition<S, R> {
    Pop(R),
    Terminate,
    Push(Box<dyn State<State = S, StateResult = R>>),
    Switch(Box<dyn State<State = S, StateResult = R>>),
}

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

/// Return value for `update` callback sent into [run] that controls the main event loop.
pub enum RunControl {
    // Quit the run loop.
    Quit,
    // Call `update` again next frame.
    Update,
    // Wait for an input event before the next update; this will likely draw the mode before
    // waiting.
    WaitForEvent,
}
