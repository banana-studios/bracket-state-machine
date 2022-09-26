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
    pub wait_for_event: bool,
    pub pop_result: Option<R>,
    pub active_mouse_pos: Point,
    pub states: Vec<Box<dyn State<State = S, StateResult = R>>>,

    #[cfg(not(feature = "self-logic"))]
    global_tick_fn: Option<Box<dyn FnMut(&mut BTerm, &mut S)>>,
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

            #[cfg(not(feature = "self-logic"))]
            global_tick_fn: None,
        }
    }

    /// Set a function to be called every tick, before the current state's `update` function.
    /// This tick fn will run even if the state machine is waiting for an event.
    #[cfg(not(feature = "self-logic"))]
    pub fn add_global_tick_fn<F>(&mut self, global_tick_fn: F)
    where
        F: FnMut(&mut BTerm, &mut S) + 'static + Sized,
    {
        self.global_tick_fn = Some(Box::new(global_tick_fn));
    }
}

//////////////////////////////////////////////////////////////////////////////
// Internals
//////////////////////////////////////////////////////////////////////////////
#[cfg(not(feature = "self-logic"))]
impl<S, R> StateMachine<S, R> {
    fn clear_consoles(&mut self, term: &mut BTerm) {
        if let Some(top_state) = self.states.last_mut() {
            top_state.clear(term, &self.state);
        }
    }

    fn internal_tick(&mut self, ctx: &mut BTerm) -> RunControl {
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

#[cfg(not(feature = "self-logic"))]
impl<S: 'static, R: 'static> GameState for StateMachine<S, R> {
    fn tick(&mut self, ctx: &mut BTerm) {
        if ctx.quitting {
            ctx.quit();
        }

        // Global tick fn ticks every frame no matter if the state machine is waiting for an event.
        if let Some(func) = &mut self.global_tick_fn {
            func(ctx, &mut self.state);
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
