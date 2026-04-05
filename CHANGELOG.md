## [0.2.0] - 2026-04-05

### 🚀 Features

- Support generation of truth tables
- Add a TUI for propositional logic
- Show a classification of the formula regarding SAT, tautology, contradiction
- First stab at parser for set theory
- First stab at eval for set theory
- Improve parser
- Implement some basic TUI operations for set theory
- Extend the element parser to numbers and {}
- Start supporting parsing of non-empty inner sets
- Support any nested set scenario, nested operators still todo
- Add support for set differences
- First stab at assignments
- Support multiline set theory terms
- Handle unknown identifiers gracefully
- Support declaration of a universe set UNI and the complement operation
- Support expressions in set definitions
- Add support for settheory
- *(settheory)* Add subset, strict subset operatos
- *(set theory)* Support cardinality operator
- *(set theory)* Add support for cartesian product
- *(propositional logic)* Show row index, row count and vars count
- *(propositional logic)* Offer filters, show stats
- *(propositional logic)* Support xor
- *(ui)* Eval on F5 in propositional logic, too
- *(ui)* Some sizing improvements
- *(ui)* Show key bindings
- *(ui)* Show syntax help
- *(ui)* Offer helper hints about usage and syntax
- *(set theory)* Support equality operator
- *(set theory)* Support size literals
- *(dfa)* Support DFA
- *(dfa)* Add some trace wrappers around parser combinators
- *(dfa)* Parse whole DFA definition
- *(dfa)* Improve state parser
- *(dfa)* Improve dfa parser
- *(dfa)* Improve naming
- *(dfa)* Improve dfa parser
- *(dfa)* Show keybindings, help and parse errors
- *(dfa)* Put a default example definition into the textarea
- *(dfa)* Improve error handling
- *(dfa)* Make fields of RunningDfa pub(crate)
- *(dfa)* Support DFA
- *(ui)* Improve tab navigation
- *(dfa)* Implement a history of state transitions
- *(dfa)* Make transitions scrollable

### 🐛 Bug Fixes

- Use True symbol ⊨ instead of single turnstile
- *(parser)* Don't ignore dangling input after succesful expression was parsed
- *(ui)* Start with set theory as it's more basic
- *(set theory)* Handle empty input
- *(propositional logic)* Reset filter on eval
- *(ui)* Add help key binding hint
- *(ui)* Add missing scroll key bindings, remove margin
- *(dfa)* Fix empty/non-empty sets
- *(dfa)* Use HashMap for delta
- *(dfa)* Wire up dfa logic to TUI
- *(dfa)* Simplify non-deterministic check
- *(dfa)* Restrict word input to single line, eval on KeyCode::Enter (#42)

### 🚜 Refactor

- Use HashSet as internal type
- Simplify parser
- Simplify blocks for textareas

### 📚 Documentation

- Add README.md
- Add LICENSE file
- Document public items
- Add some real content to the README.md
- Add README.md
- Use img tags for screenshots
- Update screenshots
- Add DFA part to README.md
- Update screenshots

## [0.1.3] - 2026-03-20

### 🚀 Features

- Implement basic propositional logic
- First stab at parser
- Parse not
- Parse expressions
- Parse bool literals
- Support equivalence
- Support implication
- Implement collect_vars() and all_assignments()
- Support generation of truth tables
- Add a TUI for propositional logic
- Show a classification of the formula regarding SAT, tautology, contradiction
- First stab at TUI part of set theory
- First stab at parser for set theory
- First stab at eval for set theory
- Improve parser
- Implement some basic TUI operations for set theory
- Extend the element parser to numbers and {}
- Start supporting parsing of non-empty inner sets
- Support any nested set scenario, nested operators still todo
- Add support for set differences
- First stab at assignments
- Support multiline set theory terms
- Handle unknown identifiers gracefully
- Support declaration of a universe set UNI and the complement operation
- Support expressions in set definitions
- *(settheory)* Add subset, strict subset operatos
- *(set theory)* Support cardinality operator
- *(set theory)* Add support for cartesian product
- *(propositional logic)* Show row index, row count and vars count
- *(propositional logic)* Offer filters, show stats
- *(propositional logic)* Support xor
- *(ui)* Eval on F5 in propositional logic, too
- *(ui)* Some sizing improvements
- *(ui)* Show key bindings
- *(ui)* Show syntax help
- *(set theory)* Support equality operator
- *(set theory)* Support size literals

### 🐛 Bug Fixes

- Use True symbol ⊨ instead of single turnstile
- *(parser)* Don't ignore dangling input after succesful expression was parsed
- *(ui)* Start with set theory as it's more basic
- *(set theory)* Handle empty input
- *(propositional logic)* Reset filter on eval
- *(ui)* Add help key binding hint
- *(ui)* Add missing scroll key bindings, remove margin

### 💼 Other

- Add .woodpecker.yaml

### 🚜 Refactor

- Make Expr enum more ergonomic
- Use HashSet as internal type
- Simplify parser

### 📚 Documentation

- Add README.md
- Add LICENSE file
- Document public items
- Add some real content to the README.md
- Add README.md
- Use img tags for screenshots

### ⚙️ Miscellaneous Tasks

- Cleanup
- Add github mirror README.md
