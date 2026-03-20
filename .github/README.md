# theotui

***Please note: This is only a mirror repository. Development takes place on [codeberg](https://codeberg.org/dawe/theotui).***

---

A TUI for various topics from theoretical computer science, implemented in Rust with [Ratatui](https://ratatui.rs/).  
It aims to help learning these topics by offering simple ways of exploration and experimentation.  
Currently the following topics are implemented:
- set theory
- propositional logic

More will follow.  
The core logic, without the TUI part, can be used through the crate `theoinf`.

## set theory

All the usual operations of naive set theory are implemented.  
Press `F1` to toggle the help next to the editor.  
Press `F5` to let your terms be evaluated.

<img src="https://codeberg.org/dawe/theotui/raw/commit/0ad9565acd7e5ef3ec89de7b00098a97e39d7003/theotui/images/set_theory.png" alt="set theory">

## propositional logic

All the usual operations of propositional logic are implemented.  
Press `F1` to toggle the help next to the editor.  
Press `F5` or `Enter` to let your formula be evaluated.  
The truth table can be filtered with `Ctrl-t`/`Ctrl-f` to only show the assignments resulting in true or false.

<img src="https://codeberg.org/dawe/theotui/raw/commit/0ad9565acd7e5ef3ec89de7b00098a97e39d7003/theotui/images/propositional_logic.png" alt="propositional logic">

