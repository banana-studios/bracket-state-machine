use std::fmt::format;

use bracket_lib::prelude::*;
use bracket_state_machine::prelude::*;

pub type TransitionResult = (StateTransition<Game, ModeResult>, TransitionControl);

#[derive(Default)]
struct TitleState;

#[derive(Clone)]
pub enum TitleResult {
    Quit,
}

#[derive(Clone)]
pub enum ModeResult {
    TitleResult(TitleResult),
    PausedResult(PausedResult),
}

impl State for TitleState {
    type State = Game;
    type StateResult = ModeResult;

    fn update(
        &mut self,
        term: &mut BTerm,
        state: &mut Self::State,
        _pop_result: &Option<Self::StateResult>,
    ) -> TransitionResult {
        if let Some(key) = term.key {
            match key {
                VirtualKeyCode::Escape => {
                    return (StateTransition::Terminate, TransitionControl::Update);
                }
                VirtualKeyCode::Space => {
                    return (
                        StateTransition::Push(PausedState.boxed()),
                        TransitionControl::Update,
                    );
                }
                VirtualKeyCode::Up => {
                    state.num += 1;
                }
                VirtualKeyCode::Down => {
                    state.num -= 1;
                }
                _ => {}
            }

            if key == VirtualKeyCode::Space {
                return (
                    Transition::Push(Box::new(PausedState)),
                    TransitionControl::Update,
                );
            }
        }

        (Transition::Stay, TransitionControl::Update)
    }

    fn render(&mut self, term: &mut BTerm, state: &mut Self::State, active: bool) {
        term.print(
            1,
            2,
            "Press [UP] to increment counter and [DOWN] to decrement counter.",
        );
        term.print(1, 4, "Press [ESC] to close & [SPACE] to transition");
        term.print(1, 6, format!("State: {:?}", state));
    }

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

struct PausedState;

#[derive(Clone)]
pub enum PausedResult {
    Resume,
    Quit,
}

impl State for PausedState {
    type State = Game;
    type StateResult = ModeResult;

    fn update(
        &mut self,
        term: &mut BTerm,
        state: &mut Self::State,
        _pop_result: &Option<ModeResult>,
    ) -> TransitionResult {
        if let Some(key) = term.key {
            if key == VirtualKeyCode::Space {
                return (
                    Transition::Pop(ModeResult::PausedResult(PausedResult::Resume)),
                    TransitionControl::Update,
                );
            }
        }

        (Transition::Stay, TransitionControl::Update)
    }

    fn render(&mut self, term: &mut BTerm, state: &mut Self::State, active: bool) {
        term.print_centered(0, format!("PAUSED {:?}", state));
    }
}

#[derive(Debug)]
pub struct Game {
    pub num: i32,
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("State Machine Sample")
        .with_dimensions(100, 70)
        .with_fps_cap(24.0)
        .build()
        .expect("failed to build a BTerm");

    let machine = StateMachine::new(Game { num: 0 }, TitleState::default());
    main_loop(context, machine)
}
