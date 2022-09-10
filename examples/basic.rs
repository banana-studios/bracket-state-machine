use bracket_lib::prelude::*;
use bracket_state_machine::prelude::*;

pub type TransitionResult = (StateTransition<Game, ModeResult>, TransitionUpdate);

#[derive(Default)]
struct TitleState;
struct PausedState;

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
        state: &Self::State,
        _pop_result: &Option<Self::StateResult>,
        dt: DeltaTime,
    ) -> TransitionResult {
        if let Some(key) = term.key {
            if key == VirtualKeyCode::Escape {
                return (Some(Transition::Terminate), TransitionUpdate::Immediate);
            }

            if key == VirtualKeyCode::Space {
                return (
                    Some(Transition::Push(Box::new(PausedState))),
                    TransitionUpdate::Update,
                );
            }
        }

        (None, TransitionUpdate::Update)
    }

    fn draw(&self, term: &mut BTerm, state: &Self::State, active: bool) {
        term.print(1, 1, "Hello, world!");
    }

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
        state: &Self::State,
        _pop_result: &Option<ModeResult>,
        _dt: DeltaTime,
    ) -> TransitionResult {
        if let Some(key) = term.key {
            if key == VirtualKeyCode::Space {
                return (
                    Some(Transition::Pop(ModeResult::PausedResult(
                        PausedResult::Resume,
                    ))),
                    TransitionUpdate::Update,
                );
            }
        }

        (None, TransitionUpdate::Update)
    }

    fn draw(&self, term: &mut BTerm, state: &Self::State, active: bool) {
        term.print_centered(0, "PAUSED");
    }
}

pub struct Game {}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("State Machine Sample")
        .with_fps_cap(24.0)
        .build()
        .expect("failed to build a BTerm");

    let machine = StateMachine::new(Game {}, TitleState::default());
    main_loop(context, machine)
}
