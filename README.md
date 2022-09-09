# Welcome to bracket-state-machine

![](https://github.com/lecoqjacob/bracket-state-machine/workflows/Rust/badge.svg)

## Using `bracket-state-machine`

In your `Cargo.toml` file, include:

```toml
[dependencies]
bracket-state-machine = "0.1"
```

If you wish to ultilize the git branch of `bracket-terminal`, just use the `bracket-main` branch

```toml
[dependencies]
bracket-state-machine = {git = "https://github.com/lecoqjacob/bracket-state-machine", branch = "bracket-main"}
```


## Feature Flags

This crate supports all feature flags of `bracket-terminal`

## Examples

* `basic` basic demonstartion of the state machine in action. It switches between two states.