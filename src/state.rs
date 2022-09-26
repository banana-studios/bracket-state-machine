use bracket_lib::prelude::*;

pub type StateTransition<S, R> = Transition<S, R>;
pub type StateReturn<S, R> = (StateTransition<S, R>, TransitionControl);

pub trait State {
    type State: Sized;
    type StateResult;

    #[must_use = "it may trigger a state change"]
    fn update(
        &mut self,
        term: &mut BTerm,
        state: &mut Self::State,
        pop_result: &Option<Self::StateResult>,
    ) -> StateReturn<Self::State, Self::StateResult>;

    fn render(&mut self, term: &mut BTerm, state: &mut Self::State, active: bool);

    #[allow(unused_variables)]
    #[inline]
    fn clear(&self, term: &mut BTerm, state: &Self::State) {
        BACKEND_INTERNAL
            .lock()
            .consoles
            .iter_mut()
            .for_each(|c| c.console.cls());
    }

    #[inline]
    fn draw_behind(&self) -> bool {
        true
    }

    #[inline]
    fn boxed(self) -> Box<dyn State<State = Self::State, StateResult = Self::StateResult>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
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

pub enum Transition<S, R> {
    Stay,
    Pop(R),
    Terminate,
    Push(Box<dyn State<State = S, StateResult = R>>),
    Switch(Box<dyn State<State = S, StateResult = R>>),
}

/// Desired behavior for the next update, to be returned from an `update` call.
#[derive(Debug)]
pub enum TransitionControl {
    /// Run the next update immediately, without waiting for the next frame.
    Immediate,
    /// Wait a frame before the next update; this will likely draw the mode for a frame.
    Update,
    /// Wait for an input event before the next update; this will likely draw the mode before
    /// waiting.
    WaitForEvent,
}

#[allow(clippy::type_complexity)]
pub struct StateMachine<S, R> {
    pub state: S,
    pub pop_result: Option<R>,
    pub states: Vec<Box<dyn State<State = S, StateResult = R>>>,
}

impl<S, R> StateMachine<S, R> {
    // TODO implement From<State>
    /// creates a state machine with an initial state
    pub fn new<T: State<State = S, StateResult = R> + 'static>(
        system_state: S,
        init_state: T,
    ) -> Self {
        StateMachine {
            pop_result: None,
            state: system_state,
            states: vec![Box::new(init_state)],
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// Internals
//////////////////////////////////////////////////////////////////////////////
impl<S, R> StateMachine<S, R> {
    fn clear_consoles(&mut self, term: &mut BTerm) {
        if let Some(top_state) = self.states.last_mut() {
            top_state.clear(term, &self.state);
        }
    }

    pub fn update(&mut self, ctx: &mut BTerm) -> RunControl {
        while !self.states.is_empty() {
            let (transition, transition_update) = {
                let top_mode = self.states.last_mut().unwrap();
                top_mode.update(ctx, &mut self.state, &self.pop_result)
            };

            self.pop_result = None;
            match transition {
                Transition::Stay => {}
                Transition::Switch(state) => {
                    self.states.pop();
                    self.states.push(state);
                }
                Transition::Push(state) => {
                    self.states.push(state);
                }
                Transition::Pop(state_result) => {
                    self.pop_result = Some(state_result);
                    self.states.pop();
                }
                Transition::Terminate => {
                    self.states.clear();
                }
            }

            // Draw modes in the stack from the bottom-up.
            if !self.states.is_empty() && !matches!(transition_update, TransitionControl::Immediate)
            {
                let draw_from = self
                    .states
                    .iter()
                    .rposition(|mode| !mode.draw_behind())
                    .unwrap_or(0);

                let top = self.states.len().saturating_sub(1);

                self.clear_consoles(ctx);

                for mode in self.states.iter_mut().skip(draw_from).take(top) {
                    mode.render(ctx, &mut self.state, false);
                }

                // Draw top mode with `active` set to `true`.
                self.states[top].render(ctx, &mut self.state, true);

                render_draw_buffer(ctx).expect("Render draw buffer error");
            }

            match transition_update {
                TransitionControl::Immediate => (),
                TransitionControl::Update => return RunControl::Update,
                TransitionControl::WaitForEvent => return RunControl::WaitForEvent,
            }
        }

        RunControl::Quit
    }
}
