# theotui

***Please note: This is only a mirror repository. Development takes place on [codeberg](https://codeberg.org/dawe/theotui).***

---

A TUI for various topics from theoretical computer science, implemented in Rust with [Ratatui](https://ratatui.rs/).  
It aims to help learning these topics by offering simple ways of exploration and experimentation.  
Currently the following topics are implemented:
- set theory
- propositional logic
- deterministic finite automata

More will follow.  
The core logic, without the TUI part, can be used through the crate `theoinf`.

## set theory

All the usual operations of naive set theory are implemented.  
Press `F1` to toggle the help next to the editor.  
Press `F5` to let your terms be evaluated.

<img src="https://codeberg.org/dawe/theotui/raw/commit/4bbf924245580c5fe66b892e08ffeeacb0d70695/theotui/images/set_theory.png" alt="set theory">

## propositional logic

All the usual operations of propositional logic are implemented.  
Press `F1` to toggle the help next to the editor.  
Press `F5` or `Enter` to let your formula be evaluated.  
The truth table can be filtered with `Ctrl-t`/`Ctrl-f` to only show the assignments resulting in true or false.  
The CNF and DNF are constructed if possible.

<img src="https://codeberg.org/dawe/theotui/raw/commit/715128adf26fdc4209ad70f72353919297efbcfc/theotui/images/propositional_logic.png" alt="propositional logic">

## deterministic finite automata (DFA)

A `DFA` is defined with the usual 5 parts:

- `Sigma`, the alphabet
- `S`, the set of states
- `start`, the starting state
- `F`, the set of accepting states
- `delta`, the set of state transitions

A single transition is a tuple of 3: `(current_state, symbol, next_state)`  
Press `F1` to toggle the help next to the editor.  
Press `F5` or `Enter` in the `Word` input to let your word be checked for acceptance.  

<img src="https://codeberg.org/dawe/theotui/raw/commit/93740bfa3207594639035d4e87eaa0f896c118ba/theotui/images/dfa.png" alt="dfa">

## Installation

```shell
cargo install theotui
```

