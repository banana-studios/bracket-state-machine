use crate::state::{State, TransitionUpdate};
use bracket_lib::prelude::*;
use std::time::Duration;

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

pub struct StateMachine<S, R> {
    state: S,
    wait_for_event: bool,
    pop_result: Option<R>,
    active_mouse_pos: Point,
    states: Vec<Box<dyn State<State = S, StateResult = R>>>,
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
            wait_for_event: false,
            active_mouse_pos: Point::zero(),
            states: vec![Box::new(init_state)],
        }
    }

    fn clear_consoles(&mut self, term: &mut BTerm) {
        if let Some(top_state) = self.states.last_mut() {
            top_state.clear(&self.state, term);
        }
    }

    fn internal_tick(&mut self, ctx: &mut BTerm) -> RunControl {
        while !self.states.is_empty() {
            let (transition, transition_update) = {
                let top_mode = self.states.last_mut().unwrap();
                top_mode.update(
                    ctx,
                    &self.state,
                    &self.pop_result,
                    Duration::from_millis(ctx.frame_time_ms as u64),
                )
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
            if !self.states.is_empty() && !matches!(transition_update, TransitionUpdate::Immediate)
            {
                let draw_from = self
                    .states
                    .iter()
                    .rposition(|mode| !mode.is_transparent())
                    .unwrap_or(0);

                let top = self.states.len().saturating_sub(1);

                self.clear_consoles(ctx);

                // Draw non-top modes with `active` set to `false`.
                for mode in self.states.iter_mut().skip(usize::max(draw_from, 1)) {
                    mode.render(ctx, &self.state, false);
                }

                // Draw top mode with `active` set to `true`.
                self.states[top].render(ctx, &self.state, true);

                render_draw_buffer(ctx).expect("Render draw buffer error");
            }

            match transition_update {
                TransitionUpdate::Immediate => (),
                TransitionUpdate::Update => return RunControl::Update,
                TransitionUpdate::WaitForEvent => return RunControl::WaitForEvent,
            }
        }

        RunControl::Quit
    }
}

impl<S: 'static, R: 'static> GameState for StateMachine<S, R> {
    fn tick(&mut self, ctx: &mut BTerm) {
        if ctx.quitting {
            ctx.quit();
        }

        if self.wait_for_event {
            let new_mouse = ctx.mouse_point();

            // Handle Keys & Mouse Clicks
            if ctx.key.is_some() || ctx.left_click {
                self.wait_for_event = false;
            }

            // Handle Mouse Movement
            if new_mouse != self.active_mouse_pos {
                self.wait_for_event = false;
                self.active_mouse_pos = new_mouse;
            }
        } else {
            self.active_mouse_pos = ctx.mouse_point();

            match self.internal_tick(ctx) {
                RunControl::Update => {}
                RunControl::Quit => ctx.quit(),
                RunControl::WaitForEvent => self.wait_for_event = true,
            }
        }
    }
}
