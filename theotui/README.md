# theotui

A TUI for various topics from theoretical computer science, implemented in Rust with [Ratatui](https://ratatui.rs/).  
It aims to help learning these topics by offering simple ways of exploration and experimentation.  
Currently the following topics are implemented:
- set theory
- propositional logic
- deterministic finite automata
- type-3 grammars
- type-2 grammars

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
With the argument `--dfa file.txt` a stored definition can be read from the given file at the start of theotui.  
Here's a complete definition example:
```
Sigma = { 'a', 'b' }
S = { s0, s1, s2 }
start = s0
F = { s2 }
delta = { (s0, 'a', s1), (s1, 'b', s2) }
```

<img src="https://codeberg.org/dawe/theotui/raw/commit/218b531040890c543b7e8782858a988c1e23e85a/theotui/images/dfa.png" alt="dfa">

## type-3 grammars

A `Type-3 Grammar` is defined with the usual 4 parts:

- `V`, the set of non-terminals
- `Sigma`, the set of terminals
- `P`, the set of production rules
- `S`, the start non-terminal

A single right-regular production rule has one of the following three forms:  

- `T -> 'aT'`  
- `T -> 'a'`  
- `T -> ''` here the empty word epsilon is denoted as `''`

A single left-regular production rule has one of the following three forms:  

- `T -> 'Ta'`  
- `T -> 'a'`  
- `T -> ''` here the empty word epsilon is denoted as `''`

Press `F1` to toggle the help next to the editor.  
Press `F5` or `Enter` in the `Word` input to let your word be checked for a possible production.  
With the argument `--t3g file.txt` a stored definition can be read from the given file at the start of theotui.  
Here's a complete definition example:
```
V = { S, T }
Sigma = { 'a', 'b' }
P = { S -> 'aT', T -> 'b', T -> 'bT', T -> '' }
S = S
```

<img src="https://codeberg.org/dawe/theotui/raw/commit/eabd98ad2ca3b542914f19bf863c7ab0a58113b8/theotui/images/type3grammar.png" alt="type-3 grammar">

## type-2 grammars

A `Type-2 Grammar` is defined with the usual 4 parts:

- `V`, the set of non-terminals
- `Sigma`, the set of terminals
- `P`, the set of production rules
- `S`, the start non-terminal

A production rule has one of the following forms:  

- `T -> '(Sigma ∪ V)'*`  
- `T -> ''` here the empty word epsilon is denoted as `''`

Press `F1` to toggle the help next to the editor.  
Press `F5` or `Enter` in the `Word` input to let your word be checked for a possible production.  
With the argument `--t2g file.txt` a stored definition can be read from the given file at the start of theotui.  
Here's a complete definition example:
```
V = { S }
Sigma = { '(', ')' }
P = { S -> '(S)', S -> '()', S -> '' }
S = S"
```

<img src="https://codeberg.org/dawe/theotui/raw/commit/151fd2ca1c731031cd1f2d9ee28b5563bdaed997/theotui/images/type2grammar.png" alt="type-2 grammar">

## Usage

```shell
Usage: theotui [OPTIONS]
Options:
  --dfa file        Read DFA definition from file
  --t2g file        Read Type-2 Grammar definition from file
  --t3g file        Read Type-3 Grammar definition from file
  --help            Print help
```

## Installation

```shell
cargo install theotui
```
