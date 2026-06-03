use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
    sync::LazyLock,
};

pub type Terminal = String;
pub type NonTerminal = String;
pub type Rhs = Vec<String>;
pub type ProductionRule = (NonTerminal, Rhs);

/// The empty word epsilon
static EPSI: LazyLock<Rhs> = LazyLock::new(|| vec!["".to_string()]);

pub mod parser {
    use std::collections::HashSet;

    use winnow::{
        ModalResult, Parser,
        ascii::multispace0,
        combinator::{alt, cut_err, delimited, separated, trace},
        error::{ContextError, ErrMode, StrContext},
        stream::AsChar,
        token::take_while,
    };

    use crate::{
        parser_utils::whitespace_wrapped,
        type2grammar::{EPSI, NonTerminal, ProductionRule, Terminal, Type2Grammar},
    };

    /// Expressions of the [Type2Grammar] definition.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Expr {
        NonTerms(Vec<String>),
        Sigma(Vec<String>),
        Productions(Vec<ProductionRule>),
        Start(String),
    }

    /// Parses a non-terminals definition like `V = { S, T, W }`
    pub fn parse_v_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("V");
        let equals = whitespace_wrapped("=");
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(1.., nonterminal_name(), separator);
        let setp = delimited(
            delimited(multispace0, "{", multispace0),
            comma_sep_list,
            delimited(multispace0, cut_err("}"), multispace0),
        );
        let mut decl = (identifier, equals, setp)
            .context(StrContext::Label("V definition"))
            .map(|(_, _, x): (_, _, Vec<&str>)| {
                Expr::NonTerms(x.iter().map(|s| s.to_string()).collect())
            });
        decl.parse_next(input)
    }

    /// Parses a Sigma definition like `Sigma = { 'a', 'b', 'c' }`
    pub fn parse_sigma_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("Sigma");
        let equals = whitespace_wrapped("=");
        let element = delimited("'", terminal_symbol(), cut_err("'"));
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(1.., element, separator);
        let setp = delimited(
            delimited(multispace0, "{", multispace0),
            comma_sep_list,
            delimited(multispace0, cut_err("}"), multispace0),
        );
        let mut decl = (identifier, equals, setp)
            .context(StrContext::Label("Sigma definition"))
            .map(|(_, _, x): (_, _, Vec<&str>)| {
                Expr::Sigma(x.iter().map(|s| s.to_string()).collect())
            });
        decl.parse_next(input)
    }

    pub fn nonterminal_name<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1..=1, |c: char| c.is_alpha() && c.is_uppercase())
    }

    pub fn terminal_symbol<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(1..=1, |c: char| {
            !c.is_whitespace() && !c.is_uppercase() && !c.is_control() && c != '\''
        })
    }

    /// Parses the right side of a production
    pub fn production_rhs<'s>() -> impl Parser<&'s str, &'s str, ErrMode<ContextError>> {
        take_while(0.., |c: char| {
            !c.is_whitespace() && !c.is_control() && c != '\''
        })
    }

    /// Parses a production rule like `S -> 'aTa'`
    pub fn production_rule<'s>()
    -> impl Parser<&'s str, (&'s str, Vec<String>), ErrMode<ContextError>> {
        let element = delimited("'", production_rhs(), cut_err("'"));
        let tuple = (nonterminal_name(), whitespace_wrapped("->"), element);
        trace(
            "production_rule",
            tuple.map(|(nt, _arrow, rhs)| {
                (
                    nt,
                    if rhs.is_empty() {
                        EPSI.clone()
                    } else {
                        rhs.chars().map(|c| c.to_string()).collect()
                    },
                )
            }),
        )
    }

    /// Parses a production set like `{ S -> 'aT', T -> 'b' }`
    pub fn production_set<'s>()
    -> impl Parser<&'s str, Vec<(&'s str, Vec<String>)>, ErrMode<ContextError>> {
        let separator = whitespace_wrapped(",");
        let comma_sep_list = separated(0.., production_rule(), separator);
        trace(
            "production_set",
            delimited(
                delimited(multispace0, "{", multispace0),
                comma_sep_list,
                delimited(multispace0, cut_err("}"), multispace0),
            ),
        )
    }

    /// Parse a productions definition like `P = { (S, 'aT'), (T, 'b') }`
    pub fn parse_productions_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("P");
        let equals = whitespace_wrapped("=");
        (identifier, equals, production_set())
            .context(StrContext::Label("P definition"))
            .map(|(_, _, prods)| {
                let prods: Vec<(String, Vec<String>)> = prods
                    .iter()
                    .map(|(l, r)| (l.to_string(), r.clone()))
                    .collect();
                Expr::Productions(prods)
            })
            .parse_next(input)
    }

    /// Parse a start symbol definition like `S = S`
    pub fn parse_start_nonterminal_definition(input: &mut &str) -> ModalResult<Expr> {
        let identifier = whitespace_wrapped("S");
        let equals = whitespace_wrapped("=");
        let state = delimited(multispace0, nonterminal_name(), multispace0);
        (identifier, equals, state)
            .context(StrContext::Label("S definition"))
            .map(|(_, _, x): (&str, &str, &str)| Expr::Start(x.to_string()))
            .parse_next(input)
    }

    /// Parse a [Type2Grammar] definition (V, Sigma, P, S)
    pub fn parse_t2grammar_definition(input: &str) -> Result<Type2Grammar, String> {
        let mut input = input;
        let mut nonterminals: Option<HashSet<NonTerminal>> = None;
        let mut sigma: Option<HashSet<Terminal>> = None;
        let mut productions: Option<HashSet<(NonTerminal, Vec<String>)>> = None;
        let mut start: Option<NonTerminal> = None;

        let mut alt_parser = alt((
            parse_sigma_definition,
            parse_v_definition,
            parse_productions_definition,
            parse_start_nonterminal_definition,
        ));

        for _ in 0..4 {
            let r = alt_parser.parse_next(&mut input);
            match r {
                Ok(expr) => match expr {
                    Expr::NonTerms(nonterms) => {
                        nonterminals = Some(nonterms.into_iter().collect());
                    }
                    Expr::Sigma(terms) => {
                        sigma = Some(terms.into_iter().collect());
                    }
                    Expr::Productions(prods) => {
                        productions = Some(prods.into_iter().collect());
                    }
                    Expr::Start(s) => start = Some(s),
                },
                Err(s) => return Err(s.to_string()),
            }
        }

        if !input.trim().is_empty() {
            Err("bad definition".into())
        } else if let (Some(nonterminals), Some(sigma), Some(productions), Some(start)) =
            (nonterminals, sigma, productions, start)
        {
            Type2Grammar::new(nonterminals, sigma, productions, start)
        } else {
            Err("Incomplete definition".into())
        }
    }
}

/// Defines a Type-2 Grammar
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type2Grammar {
    pub(crate) nonterminals: HashSet<NonTerminal>,
    pub(crate) sigma: HashSet<Terminal>,
    pub(crate) productions: HashSet<ProductionRule>,
    pub(crate) start: NonTerminal,
}

impl Display for Type2Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str("V = { ");
        let mut nts: Vec<_> = self.nonterminals().iter().map(|s| s.as_str()).collect();
        nts.sort();
        s.push_str(&nts.join(", "));
        s.push_str(" }\n");

        s.push_str("Sigma = { ");
        let mut sigma: Vec<_> = self.sigma().iter().map(|s| format!("'{s}'")).collect();
        sigma.sort();
        s.push_str(&sigma.join(", "));
        s.push_str(" }\n");

        s.push_str("P = { ");
        let mut p: Vec<_> = self
            .productions()
            .iter()
            .map(|(lhs, rhs)| {
                let rhs = rhs.join("");
                format!("{lhs} -> '{rhs}'")
            })
            .collect();
        p.sort();
        s.push_str(&p.join(", "));
        s.push_str(" }\n");

        s.push_str(format!("S = {}", self.start()).as_str());

        writeln!(f, "{}", s)
    }
}

type Edge = (NonTerminal, NonTerminal);
type Graph = (HashSet<NonTerminal>, HashSet<Edge>);
type Path = Vec<NonTerminal>;

impl Type2Grammar {
    /// Constructs a valid [Type2Grammar]
    pub fn new(
        nonterminals: HashSet<NonTerminal>,
        sigma: HashSet<Terminal>,
        productions: HashSet<(NonTerminal, Vec<String>)>,
        start: NonTerminal,
    ) -> Result<Self, String> {
        if !nonterminals.contains(&start) {
            return Err("start must be an element of V.".into());
        }

        let (mut unknown_prod_nonterms, mut unknown_prod_terms): (Vec<String>, Vec<String>) =
            productions.iter().fold(
                (vec![], vec![]),
                |(mut unknown_nonterms, mut unknown_terms): (Vec<String>, Vec<_>), (nt, rhs)| {
                    if !nonterminals.contains(nt) {
                        unknown_nonterms.push(nt.to_string());
                    }

                    let terms: Vec<_> = rhs
                        .iter()
                        .filter(|s| !s.is_empty() && s.chars().all(|c| !c.is_uppercase()))
                        .collect();
                    terms.iter().for_each(|c| {
                        if !sigma.contains(&c.to_string()) {
                            unknown_terms.push(c.to_string());
                        }
                    });

                    let non_terms: Vec<_> = rhs
                        .iter()
                        .filter(|s| s.starts_with(|c: char| c.is_uppercase()))
                        .collect();
                    non_terms.iter().for_each(|c| {
                        if !nonterminals.contains(&c.to_string()) {
                            unknown_nonterms.push(c.to_string());
                        }
                    });
                    (unknown_nonterms, unknown_terms)
                },
            );

        if !unknown_prod_nonterms.is_empty() {
            unknown_prod_nonterms.sort();
            unknown_prod_nonterms.dedup();
            let s = unknown_prod_nonterms.join(", ");
            let msg = format!("The productions contain the following unknown non-terminals: {s}");
            return Err(msg);
        }

        if !unknown_prod_terms.is_empty() {
            unknown_prod_terms.sort();
            unknown_prod_terms.dedup();
            let s = unknown_prod_terms.join(", ");
            let msg =
                format!("The productions contain the following unknown terminal symbols: {s}");
            return Err(msg);
        }

        Ok(Type2Grammar {
            nonterminals,
            sigma,
            productions,
            start,
        })
    }

    /// The non-terminals of the [Type2Grammar].
    pub fn nonterminals(&self) -> &HashSet<String> {
        &self.nonterminals
    }

    /// The alphabet sigma of the [Type2Grammar].
    pub fn sigma(&self) -> &HashSet<Terminal> {
        &self.sigma
    }

    /// The start non-terminal of the [Type2Grammar].
    pub fn start(&self) -> &NonTerminal {
        &self.start
    }

    /// The productions of the [Type2Grammar].
    pub fn productions(&self) -> &HashSet<(String, Vec<String>)> {
        &self.productions
    }

    /// Creates the powerset of a set.
    fn powerset(indexes: &BTreeSet<usize>) -> BTreeSet<BTreeSet<usize>> {
        let indexes: Vec<&usize> = indexes.iter().collect();
        let mut pset: BTreeSet<BTreeSet<usize>> = BTreeSet::new();
        let upper = 2u32.pow(indexes.len() as u32);

        for bits in 0..upper {
            let mut set: BTreeSet<usize> = BTreeSet::new();
            (0..indexes.len()).for_each(|i| {
                if ((bits >> (i as u32)) & 0x01) == 0x01 {
                    set.insert(*indexes[i]);
                }
            });

            pset.insert(set);
        }
        pset
    }

    /// Introduces a new starting nonterminal S_0 and a new rule S_0 -> S.
    fn cnf_start(&self) -> Self {
        let mut c = self.clone();

        let new_start = "S_0".to_string();
        c.nonterminals.insert(new_start.clone());
        c.start = new_start.clone();
        let new_start_production = (new_start.clone(), vec![self.start().clone()]);
        c.productions.insert(new_start_production);

        c
    }

    /// Rewrites rules with nonsolitary terminals like A -> XtY.
    fn cnf_term(&self) -> Self {
        let mut new_prods = HashSet::new();
        let mut c = self.clone();

        for (lhs, rhs) in self.productions() {
            let terms: Vec<_> = rhs
                .iter()
                .enumerate()
                .filter(|(_, s)| self.sigma.contains(*s))
                .collect();
            if terms.is_empty() || rhs.len() == 1 {
                new_prods.insert((lhs.clone(), rhs.clone()));
            } else {
                // create new production
                let mut new_rhs = rhs.clone();
                for (idx, term) in terms {
                    let new_nonterm = format!("N_{term}");
                    c.nonterminals.insert(new_nonterm.clone());
                    let new_production = (new_nonterm.clone(), vec![term.clone()]);
                    new_prods.insert(new_production);
                    new_rhs[idx] = new_nonterm;
                }
                let new_production = (lhs.clone(), new_rhs);
                new_prods.insert(new_production);
            }
        }

        c.productions = new_prods;
        c
    }

    /// Rewrites rules with a right-hand side with more than 2 nonterminals like A -> BCDE.
    fn cnf_bin(&self) -> Self {
        let mut new_prods = HashSet::new();
        let mut c = self.clone();

        for (lhs, rhs) in self.productions() {
            if rhs.len() <= 2 {
                new_prods.insert((lhs.clone(), rhs.clone()));
            } else {
                for (idx, nt) in rhs[0..rhs.len() - 1].iter().enumerate() {
                    if idx == 0 {
                        let next_new_nonterm = format!("{}_{}", lhs, idx + 1);
                        c.nonterminals.insert(next_new_nonterm.clone());
                        let p = (lhs.clone(), vec![nt.clone(), next_new_nonterm]);
                        new_prods.insert(p);
                    } else if idx == rhs.len() - 2 {
                        // last rewrite
                        let new_nonterm = format!("{}_{}", lhs, idx);
                        c.nonterminals.insert(new_nonterm.clone());
                        let p = (new_nonterm, vec![nt.clone(), rhs[idx + 1].clone()]);
                        new_prods.insert(p);
                    } else {
                        let new_nonterm = format!("{}_{}", lhs, idx);
                        c.nonterminals.insert(new_nonterm.clone());
                        let next_new_nonterm = format!("{}_{}", lhs, idx + 1);
                        c.nonterminals.insert(next_new_nonterm.clone());
                        let p = (new_nonterm, vec![nt.clone(), next_new_nonterm]);
                        new_prods.insert(p);
                    }
                }
            }
        }

        c.productions = new_prods;
        c
    }

    /// Checks if a [NonTerminal] is nullable, if it can produce epsilon.
    fn is_nullable(&self, nt: &NonTerminal) -> bool {
        fn helper(
            acc: &mut HashSet<ProductionRule>,
            productions: &HashSet<ProductionRule>,
            nt: &NonTerminal,
        ) -> bool {
            let rules_of_nt: Vec<_> = productions
                .iter()
                .filter(|p| !acc.contains(p))
                .filter(|(lhs, _rhs)| lhs == nt)
                .collect();

            let produces_eps = rules_of_nt.iter().any(|(_, r)| r == &*EPSI);
            if produces_eps {
                true
            } else {
                rules_of_nt.iter().cloned().for_each(|p| {
                    acc.insert(p.clone());
                });
                rules_of_nt
                    .iter()
                    .any(|p| p.1.iter().all(|x| helper(acc, productions, x)))
            }
        }

        helper(&mut HashSet::new(), self.productions(), nt)
    }

    /// Eliminates rules like A -> epsilon with A not being the start non-terminal.
    fn cnf_del_epsilon(&self) -> Self {
        let mut c = self.clone();

        let nullables: HashSet<_> = self
            .nonterminals()
            .iter()
            .filter(|p| *p != self.start() && self.is_nullable(p))
            .collect();

        let rules_with_nullable_on_rhs: Vec<_> = self
            .productions()
            .iter()
            .filter(|(_lhs, rhs)| rhs.iter().any(|x| nullables.contains(x)))
            .collect();
        for r in rules_with_nullable_on_rhs {
            let indexes: BTreeSet<usize> =
                r.1.iter()
                    .enumerate()
                    .filter(|(_idx, s)| nullables.contains(*s))
                    .map(|x| x.0)
                    .collect();
            let powerset = Type2Grammar::powerset(&indexes);

            for ps in powerset {
                let mut new_rhs: Rhs = vec![];
                for (idx, s) in r.1.iter().enumerate() {
                    if !ps.contains(&idx) {
                        new_rhs.push(s.clone());
                    }
                }
                if !new_rhs.is_empty() {
                    let new_rule: ProductionRule = (r.0.clone(), new_rhs);
                    c.productions.insert(new_rule);
                } else {
                    let new_rule: ProductionRule = (r.0.clone(), EPSI.clone());

                    if &r.0 == self.start() {
                        c.productions.insert(new_rule);
                    } else {
                        c.productions.remove(&new_rule);
                    }
                }
            }
        }

        let eps_rules: Vec<_> = c
            .productions()
            .iter()
            .filter(|(lhs, rhs)| lhs != c.start() && rhs == &*EPSI)
            .cloned()
            .collect();
        eps_rules.iter().for_each(|r| {
            c.productions.remove(r);
        });
        c
    }

    /// Checks if a production creates epsilon.
    fn is_epsilon_rule(p: &ProductionRule) -> bool {
        p.1 == *EPSI
    }

    /// Checks if a production is a unit rule A -> B.
    fn is_unit_rule(&self, p: &ProductionRule) -> bool {
        p.1.len() == 1 && self.nonterminals().contains(&p.1[0])
    }

    /// Creates the graph (V, E) of unit rules in the grammar.
    fn graph_of_unit_rules(&self) -> Graph {
        let mut unit_rules: HashSet<(NonTerminal, NonTerminal)> = HashSet::new();

        for p in self.productions() {
            if self.is_unit_rule(p) {
                unit_rules.insert((p.0.clone(), p.1[0].clone()));
            }
        }

        let graph: Graph = (self.nonterminals.clone(), unit_rules);
        graph
    }

    /// Finds cycles in the unit graph of the grammar that start with the given [NonTerminal].
    fn find_cycles_for(graph: &Graph, start: &NonTerminal) -> HashSet<Path> {
        let mut cycles: HashSet<Path> = HashSet::new();

        // initial paths
        let mut paths: Vec<Path> = graph
            .1
            .iter()
            .filter_map(|(l, r)| {
                if l == start {
                    Some(vec![l.clone(), r.clone()])
                } else {
                    None
                }
            })
            .collect();

        while let Some(path) = paths.pop() {
            let connected: Vec<NonTerminal> = graph
                .1
                .iter()
                .filter_map(|(l, r)| {
                    let last_node: &NonTerminal = path.iter().last().unwrap();
                    if l == last_node {
                        Some(r.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for conn in connected {
                let mut new_path: Path = path.clone();
                new_path.push(conn);
                if new_path.iter().last().unwrap() == start {
                    // cycle detected
                    cycles.insert(new_path);
                } else {
                    // check for sub cycles
                    let set: HashSet<_> = HashSet::from_iter(new_path.clone());
                    if set.len() == new_path.len() {
                        paths.push(new_path);
                    }
                }
            }
        }

        cycles
    }

    /// Finds all cycles in the given graph.
    fn find_cycles(graph: &Graph) -> HashSet<Path> {
        graph
            .0
            .iter()
            .flat_map(|nt| Type2Grammar::find_cycles_for(graph, nt))
            .collect()
    }

    /// Walks the unit graph backwards from the given ending [Edge].
    fn find_backward_unit_paths(graph: &Graph, ending_edge: &Edge) -> HashSet<Path> {
        let mut paths: HashSet<Path> = HashSet::new();
        paths.insert(vec![ending_edge.0.clone(), ending_edge.1.clone()]);

        let mut stack: Vec<Edge> = graph
            .1
            .iter()
            .filter(|(_lhs, rhs)| rhs == &ending_edge.0)
            .cloned()
            .collect();

        while let Some(edge) = stack.pop() {
            let mut extended_paths: HashSet<Path> = HashSet::new();
            for path in &paths {
                if edge.1 == path[0] {
                    let mut new_path = path.clone();
                    new_path.insert(0, edge.0.clone());
                    extended_paths.insert(new_path);
                }
            }

            let old_len = paths.len();
            paths.extend(extended_paths);
            if paths.len() > old_len {
                let starts: HashSet<_> = paths.iter().map(|p| &p[0]).collect();
                graph.1.iter().for_each(|(lhs, rhs)| {
                    if starts.contains(rhs) {
                        stack.push((lhs.clone(), rhs.clone()));
                    }
                });
            }
        }

        let subpaths_to_remove: HashSet<Path> = paths
            .iter()
            .filter(|path| {
                paths
                    .iter()
                    .any(|p| p.len() > path.len() && p.ends_with(path))
            })
            .cloned()
            .collect();
        paths.difference(&subpaths_to_remove).cloned().collect()
    }

    /// Remove unit production chains and replaces them with appropriate rules by inlining.
    fn remove_unit_productions(&self) -> Self {
        let mut c = self.clone();
        let graph = c.graph_of_unit_rules();
        let non_unit_prods: HashSet<_> = self
            .productions()
            .iter()
            .filter(|p| !self.is_unit_rule(p))
            .collect();

        for (nu_lhs, nu_rhs) in non_unit_prods {
            let unit_rules_pointing_to_non_unit_rule: HashSet<&Edge> = graph
                .1
                .iter()
                .filter(|(_u_lhs, u_rhs)| u_rhs == nu_lhs)
                .collect();
            // find backwards-paths in graph
            for unit_rule_pointing_to_non_unit_rule in unit_rules_pointing_to_non_unit_rule {
                let paths = Type2Grammar::find_backward_unit_paths(
                    &graph,
                    unit_rule_pointing_to_non_unit_rule,
                );
                paths.iter().for_each(|path| {
                    // inline unit rule
                    let replacement_rule: ProductionRule = (path[0].clone(), nu_rhs.clone());
                    c.productions.insert(replacement_rule);
                    let p: Vec<_> = path.iter().collect();
                    for window in p.windows(2) {
                        let to_remove: ProductionRule =
                            (window[0].to_string(), vec![window[1].to_string()]);
                        c.productions.remove(&to_remove);
                    }
                });
            }
        }

        // remove all unit rules
        c.productions.retain(|p| !self.is_unit_rule(p));

        c
    }

    // Eliminate unit rules like A -> B, cycles and creates appropriate replacements.
    fn cnf_del_unit_rules(&self) -> Self {
        let mut c = self.clone();
        while let cycles = Type2Grammar::find_cycles(&c.graph_of_unit_rules())
            && !cycles.is_empty()
        {
            let cycle_to_remove = cycles.iter().max_by_key(|c| c.len()).unwrap();
            // remove cyclic rules
            for i in 0..cycle_to_remove.len() - 1 {
                // A_1 -> A_2 -> A_n -> A_1
                let r: ProductionRule = (
                    cycle_to_remove[i].clone(),
                    vec![cycle_to_remove[i + 1].clone()],
                );
                c.productions.remove(&r);
            }
            // replace all A_i with A_1 for i = 2,...,n in productions
            let a_1: NonTerminal = cycle_to_remove[0].clone();
            let a_i: Vec<&String> = cycle_to_remove[1..cycle_to_remove.len() - 1]
                .iter()
                .collect();
            let mut new_prods: HashSet<ProductionRule> = HashSet::new();
            for (lhs, rhs) in c.productions() {
                let new_lhs: NonTerminal = if a_i.contains(&lhs) {
                    a_1.clone()
                } else {
                    lhs.clone()
                };
                let new_rhs: Rhs = rhs
                    .iter()
                    .map(|x| {
                        if a_i.contains(&x) {
                            a_1.clone()
                        } else {
                            x.clone()
                        }
                    })
                    .collect();
                let new_rule: ProductionRule = (new_lhs, new_rhs);
                new_prods.insert(new_rule);
            }
            c.productions = new_prods;
        }

        let mut c = c.remove_unit_productions();

        // remove all superfluous nonterminals
        c.nonterminals.retain(|nt| {
            nt == &c.start
                || c.productions
                    .iter()
                    .any(|(lhs, rhs)| lhs == nt || rhs.contains(nt))
        });

        c
    }

    /// Converts a [Type2grammar] to it's Chomsky normal form
    pub fn to_cnf(&self) -> Self {
        let g = self.cnf_start();
        let g = g.cnf_term();
        let g = g.cnf_bin();
        let g = g.cnf_del_epsilon();
        g.cnf_del_unit_rules()
    }

    /// Finds all possible [ProductionRule]s for the given [NonTerminal].
    fn possible_productions(&self, nonterm_to_expand: NonTerminal) -> Vec<ProductionRule> {
        self.productions
            .iter()
            .filter(|p| p.0 == nonterm_to_expand)
            .cloned()
            .collect()
    }

    /// Processes the top of the stack top by popping and pushing accordingly.
    fn process_stack_top(
        &self,
        word: String,
        mut stack: Vec<String>,
        acc: Vec<ProductionRule>,
    ) -> Vec<(String, Vec<String>, Vec<ProductionRule>)> {
        // bail out if we are already longer that the input word or have a bad terminal
        let terms_on_stack: Vec<&String> =
            stack.iter().filter(|c| self.sigma.contains(*c)).collect();
        if terms_on_stack.len() > word.len() || terms_on_stack.iter().any(|t| !word.contains(*t)) {
            return vec![];
        }
        // bail out if we are already longer than the word and there's no epsilon rule
        if stack.len() > word.len() && !self.productions().iter().any(Type2Grammar::is_epsilon_rule)
        {
            return vec![];
        }

        match stack.pop() {
            Some(popped) => {
                if self.nonterminals.contains(&popped) {
                    let possible_productions = self.possible_productions(popped);
                    let mut stacks = vec![];
                    for p in possible_productions {
                        let mut stack_for_p = stack.clone();
                        let input_for_p = word.to_string().clone();
                        p.1.iter().rev().for_each(|c| stack_for_p.push(c.clone()));
                        let mut acc = acc.clone();
                        acc.push(p);
                        stacks.push((input_for_p, stack_for_p, acc));
                    }
                    stacks
                } else if self.sigma.contains(&popped) {
                    if word.starts_with(popped.as_str()) {
                        let word = word.replacen(popped.as_str(), "", 1);
                        vec![(word, stack, acc)]
                    } else {
                        vec![]
                    }
                } else if popped.is_empty() {
                    vec![(word, stack, acc)]
                } else {
                    panic!("unknown stack top")
                }
            }
            None => vec![],
        }
    }

    /// Tries to find a production chain that produces the given word.
    pub fn try_find_productions(&self, word: &str) -> Option<Vec<ProductionRule>> {
        if word.is_empty() && !self.productions().iter().any(Type2Grammar::is_epsilon_rule) {
            return None;
        }

        let word = word.to_string();
        let stack: Vec<String> = vec![self.start.clone()];
        let acc: Vec<ProductionRule> = vec![];
        let mut states = vec![(word, stack, acc)];
        let mut found = None;

        while found.is_none() && !states.is_empty() {
            states = states
                .into_iter()
                .flat_map(|(w, s, acc)| self.process_stack_top(w, s, acc))
                .collect();
            found = states.iter().find_map(|(w, s, acc)| {
                if w.is_empty() && s.is_empty() {
                    Some(acc.clone())
                } else {
                    None
                }
            });
        }

        found
    }
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::type2grammar::parser::Expr;

    use super::*;

    fn str_to_rhs(s: &str) -> Rhs {
        if s.is_empty() {
            EPSI.clone()
        } else {
            s.chars().map(|c| c.into()).collect()
        }
    }

    fn assert_cnf(g: &Type2Grammar) {
        let r = g.productions().iter().all(|(_lhs, rhs)| {
            if rhs.len() == 1 {
                rhs[0].is_empty() || g.sigma().contains(&rhs[0])
            } else if rhs.len() == 2 {
                g.nonterminals().contains(&rhs[0]) && g.nonterminals().contains(&rhs[1])
            } else {
                panic!("bad rhs len")
            }
        });

        assert!(r);
    }

    #[test]
    fn start_must_be_known() {
        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), str_to_rhs("a"))]),
            "X".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn production_nonterms_must_be_known() {
        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("X".into(), str_to_rhs("a"))]),
            "S".into(),
        );
        assert!(g.is_err());

        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), str_to_rhs("aX"))]),
            "S".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn production_symbols_must_be_known() {
        let g = Type2Grammar::new(
            HashSet::from(["S".into(), "T".into(), "W".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S".into(), str_to_rhs("xT"))]),
            "S".into(),
        );
        assert!(g.is_err());
    }

    #[test]
    fn nonterms_can_be_multichar_internally() {
        let r = Type2Grammar::new(
            HashSet::from(["S_0".into(), "T_0".into()]),
            HashSet::from(["a".into(), "b".into()]),
            HashSet::from([("S_0".into(), vec!["a".into(), "T_0".into()])]),
            "S_0".into(),
        );
        assert!(r.is_ok());
        let g = r.unwrap();
        let mut expected = HashSet::new();
        expected.insert(("S_0".to_string(), vec!["a".to_string(), "T_0".to_string()]));
        assert_eq!(g.productions(), &expected);
    }

    #[test]
    fn parse_sigma_works() {
        let mut s = "Sigma = { 'a' , 'b','c' } ";
        let symbols = parser::parse_sigma_definition(&mut s).unwrap();
        assert_eq!(
            symbols,
            Expr::Sigma(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn parse_empty_sigma_should_fail() {
        let mut s = "Sigma = { } ";
        let r = parser::parse_sigma_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_v_definition_works() {
        let mut s = "V = { S , T,W  } ";
        let v = parser::parse_v_definition(&mut s).unwrap();
        assert_eq!(v, Expr::NonTerms(vec!["S".into(), "T".into(), "W".into()]));
    }

    #[test]
    fn parse_empty_v_definition_should_fail() {
        let mut s = "V = {} ";
        let r = parser::parse_v_definition(&mut s);
        assert!(r.is_err());
    }

    #[test]
    fn parse_start_state_works() {
        let mut s = "S = S";
        let nt = parser::parse_start_nonterminal_definition(&mut s).unwrap();
        assert_eq!(nt, Expr::Start("S".to_string()));
    }

    #[test]
    fn production_rhs_works_for_right_regular() {
        let r = parser::production_rhs().parse_next(&mut "").unwrap();
        assert_eq!(r, "");
        let r = parser::production_rhs().parse_next(&mut "a").unwrap();
        assert_eq!(r, "a");
        let r = parser::production_rhs().parse_next(&mut "aT").unwrap();
        assert_eq!(r, "aT");
    }

    #[test]
    fn production_rhs_works_for_left_regular() {
        let r = parser::production_rhs().parse_next(&mut "").unwrap();
        assert_eq!(r, "");
        let r = parser::production_rhs().parse_next(&mut "a").unwrap();
        assert_eq!(r, "a");
        let r = parser::production_rhs().parse_next(&mut "Ta").unwrap();
        assert_eq!(r, "Ta");
    }

    #[test]
    fn parse_production_definition_works_for_right_regular() {
        let mut s = "P = { S-> 'aT', T ->'b', B -> '' }";
        let r = parser::parse_productions_definition(&mut s).unwrap();
        assert_eq!(
            r,
            Expr::Productions(vec![
                ("S".into(), str_to_rhs("aT")),
                ("T".into(), str_to_rhs("b")),
                ("B".into(), str_to_rhs(""))
            ])
        );
    }

    #[test]
    fn production_rule_works_for_left_regular() {
        let mut s = "S -> 'Ta'";
        let r = parser::production_rule().parse_next(&mut s);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), ("S", str_to_rhs("Ta")));
    }

    #[test]
    fn parse_production_definition_works_for_left_regular() {
        let mut s = "P = { S -> 'Ta', T -> 'b', B -> '' }";
        let r = parser::parse_productions_definition(&mut s).unwrap();
        assert_eq!(
            r,
            Expr::Productions(vec![
                ("S".into(), str_to_rhs("Ta")),
                ("T".into(), str_to_rhs("b")),
                ("B".into(), str_to_rhs(""))
            ])
        );
    }

    #[test]
    fn parse_t2grammar_definition_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
        let g = r.unwrap();
        assert_eq!(
            g.nonterminals(),
            &HashSet::from_iter(["S".into(), "T".into()])
        );
        assert_eq!(g.sigma(), &HashSet::from_iter(["a".into(), "b".into()]));
        assert_eq!(
            g.productions(),
            &HashSet::from_iter([
                ("S".into(), str_to_rhs("aT")),
                ("T".into(), str_to_rhs("b"))
            ])
        );
        assert_eq!(g.start, "S");
    }

    #[test]
    fn parse_left_regular_t2grammar_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'Ta' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_t2grammar_with_left_right_mixed_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_t2grammar_definition_with_missing_but_duplicated_parts_should_fail() {
        let s = "
    Sigma = { 'a', 'b' }
    V = { S, T, W }
    Sigma = { 'a', 'b' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn productions_work_for_epsilon() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> '', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let r = g.try_find_productions("");
        assert!(r.is_some());
        assert_eq!(r, Some(vec![("S".into(), str_to_rhs(""))]));

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("").is_some());
    }

    #[test]
    fn productions_work_for_single_symbol_word() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', S -> 'a', S -> 'Sa', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let r = g.try_find_productions("a");
        assert!(r.is_some());
        assert_eq!(r, Some(vec![("S".into(), str_to_rhs("a"))]));

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("a").is_some());
    }

    #[test]
    fn productions_work_for_two_symbol_word() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let r = g.try_find_productions("ab");
        assert!(r.is_some());
        assert_eq!(
            r,
            Some(vec![
                ("S".into(), str_to_rhs("aT")),
                ("T".into(), str_to_rhs("b"))
            ])
        );

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("ab").is_some());
    }

    #[test]
    fn productions_work_for_balanced_parens() {
        let s = "
    V = { S, T }
    Sigma = { '(', ')' }
    P = { S -> '(S)', S -> '()' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("()").is_some());
        assert!(g.try_find_productions("(())").is_some());
        assert!(g.try_find_productions("((()))").is_some());

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("()").is_some());
        assert!(cnf.try_find_productions("(())").is_some());
        assert!(cnf.try_find_productions("((()))").is_some());
    }

    #[test]
    fn productions_work_for_aba() {
        let s = "
    V = { S, A, B }
    Sigma = { 'a', 'b' }
    P = { S -> 'AB', S -> 'ABA', A -> 'aA', A -> 'a', B -> 'Bb', B -> '' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("").is_none());
        assert!(g.try_find_productions(" ").is_none());
        assert!(g.try_find_productions("x").is_none());
        assert!(g.try_find_productions("a").is_some());
        assert!(g.try_find_productions("ab").is_some());
        assert!(g.try_find_productions("aa").is_some());
        assert!(g.try_find_productions("aabbaa").is_some());
        assert!(g.try_find_productions("abbbbb").is_some());

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("").is_none());
        assert!(cnf.try_find_productions(" ").is_none());
        assert!(cnf.try_find_productions("x").is_none());
        assert!(cnf.try_find_productions("a").is_some());
        assert!(cnf.try_find_productions("ab").is_some());
        assert!(cnf.try_find_productions("aa").is_some());
        assert!(cnf.try_find_productions("aabbaa").is_some());
        assert!(cnf.try_find_productions("abbbbb").is_some());
    }

    #[test]
    fn productions_work_for_arith() {
        let s = "
    V = { E, O }
    Sigma = { 'a', '(', ')', '+', '-', '*', '/' }
    P = { E -> 'a', E -> 'EOE', E -> '(E)', O -> '+', O -> '-', O -> '*', O -> '/' }
    S = E
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("a*((a-a)/a)").is_some());
        assert!(g.try_find_productions("").is_none());
        assert!(g.try_find_productions("a*").is_none());

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("a*((a-a)/a)").is_some());
        assert!(g.try_find_productions("").is_none());
        assert!(cnf.try_find_productions("a*").is_none());
    }

    #[test]
    fn productions_stop_for_impossible_word() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("aba").is_none());
    }

    #[test]
    fn productions_stop_for_word_with_invalid_terminals() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        assert!(g.try_find_productions("abx").is_none());

        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("abx").is_none());
    }

    #[test]
    fn invalid_prefix_should_fail() {
        let s = "xxx V = { S, T } Sigma = { 'a', 'b', 'c' } P = { S -> 'Tb', S -> 'Ta', T -> 'a', T -> 'Tb', T -> '' } S = S ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn invalid_postfix_should_fail() {
        let s = "V = { S, T } Sigma = { 'a', 'b', 'c' } P = { S -> 'Tb', S -> 'Ta', T -> 'a', T -> 'Tb', T -> '' } S = S xxx";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_err());
    }

    #[test]
    fn powerset_works() {
        let indexes = BTreeSet::new();
        let ps = Type2Grammar::powerset(&indexes);
        let expected = BTreeSet::from_iter(vec![BTreeSet::from_iter(vec![])]);
        assert_eq!(expected, ps);

        let indexes = BTreeSet::from_iter(vec![1]);
        let ps = Type2Grammar::powerset(&indexes);
        let expected = BTreeSet::from_iter(vec![
            BTreeSet::from_iter(vec![]),
            BTreeSet::from_iter(vec![1]),
        ]);
        assert_eq!(expected, ps);

        let indexes = BTreeSet::from_iter(vec![1, 2]);
        let ps = Type2Grammar::powerset(&indexes);
        let expected = BTreeSet::from_iter(vec![
            BTreeSet::from_iter(vec![]),
            BTreeSet::from_iter(vec![1]),
            BTreeSet::from_iter(vec![2]),
            BTreeSet::from_iter(vec![1, 2]),
        ]);
        assert_eq!(expected, ps);

        let indexes = BTreeSet::from_iter(vec![1, 2, 3]);
        let ps = Type2Grammar::powerset(&indexes);
        let expected = BTreeSet::from_iter(vec![
            BTreeSet::from_iter(vec![]),
            BTreeSet::from_iter(vec![1]),
            BTreeSet::from_iter(vec![2]),
            BTreeSet::from_iter(vec![3]),
            BTreeSet::from_iter(vec![1, 2]),
            BTreeSet::from_iter(vec![1, 3]),
            BTreeSet::from_iter(vec![2, 3]),
            BTreeSet::from_iter(vec![1, 2, 3]),
        ]);
        assert_eq!(expected, ps);
    }

    #[test]
    fn cnf_eliminate_start_works() {
        let s = "
    V = { S, T }
    Sigma = { 'a', 'b'  }
    P = { S -> 'aT', T -> 'b', T -> 'SabT', S -> '' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!("S_0", cnf.start());
        assert_eq!(
            &HashSet::from_iter(vec![
                "S_0".to_string(),
                "S".to_string(),
                "T".to_string(),
                "T_1".to_string(),
                "T_2".to_string(),
                "N_a".to_string(),
                "N_b".to_string()
            ]),
            cnf.nonterminals()
        );
        assert!(cnf.try_find_productions("ab").is_some());
    }

    #[test]
    fn cnf_bin_does_not_rewrite_already_bin_rules() {
        let s = "
    V = { A, X, Y }
    Sigma = { 'a', 'b'  }
    P = { A -> 'XY' }
    S = A
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!("S_0", cnf.start());
        assert_eq!(
            cnf.productions(),
            &HashSet::from_iter(vec![
                ("S_0".into(), str_to_rhs("XY")),
                ("A".into(), str_to_rhs("XY"))
            ])
        );
    }

    #[test]
    fn cnf_bin_works_for_one() {
        let s = "
    V = { A, X, Y, Z }
    Sigma = { 'a', 'b'  }
    P = { A -> 'XYZ' }
    S = A
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!("S_0", cnf.start());
        assert_eq!(
            cnf.productions(),
            &HashSet::from_iter(vec![
                ("S_0".into(), vec!["X".into(), "A_1".into()]),
                ("A".into(), vec!["X".into(), "A_1".into()]),
                ("A_1".into(), vec!["Y".into(), "Z".into()])
            ])
        );
    }

    #[test]
    fn cnf_bin_works_for_two() {
        let s = "
    V = { A, B, C, D, E }
    Sigma = { 'a', 'b'  }
    P = { A -> 'BCDE', A -> '' }
    S = A
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!("S_0", cnf.start());
        assert_eq!(
            &HashSet::from_iter(vec![
                ("S_0".into(), EPSI.clone()),
                ("S_0".into(), vec!["B".into(), "A_1".into()]),
                ("A".into(), vec!["B".into(), "A_1".into()]),
                ("A_1".into(), vec!["C".into(), "A_2".into()]),
                ("A_2".into(), vec!["D".into(), "E".into()])
            ]),
            cnf.productions()
        );
    }

    #[test]
    fn cnf_bin_works_for_three() {
        let s = "
    V = { A, B, C, D, E, F }
    Sigma = { 'a', 'b'  }
    P = { A -> 'BCDEF' }
    S = A
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!("S_0", cnf.start());
        assert_eq!(
            cnf.productions(),
            &HashSet::from_iter(vec![
                ("S_0".into(), vec!["B".into(), "A_1".into()]),
                ("A".into(), vec!["B".into(), "A_1".into()]),
                ("A_1".into(), vec!["C".into(), "A_2".into()]),
                ("A_2".into(), vec!["D".into(), "A_3".into()]),
                ("A_3".into(), vec!["E".into(), "F".into()])
            ])
        );
    }

    #[test]
    fn cnf_bin_works_for_four() {
        let s = "
    V = { A, B, C, D, E, F, G }
    Sigma = { 'a', 'b'  }
    P = { A -> 'BCDEFG' }
    S = A
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!("S_0", cnf.start());
        assert_eq!(
            cnf.productions(),
            &HashSet::from_iter(vec![
                ("S_0".into(), vec!["B".into(), "A_1".into()]),
                ("A".into(), vec!["B".into(), "A_1".into()]),
                ("A_1".into(), vec!["C".into(), "A_2".into()]),
                ("A_2".into(), vec!["D".into(), "A_3".into()]),
                ("A_3".into(), vec!["E".into(), "A_4".into()]),
                ("A_4".into(), vec!["F".into(), "G".into()])
            ])
        );
    }

    #[test]
    fn cnf_del_epsilon_works() {
        let s = "
    V = { S, A, B, C, D, E, F }
    Sigma = { 'a', 'b'  }
    P = { S -> '' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);

        assert_eq!(1, cnf.productions().len());
        let prod = ("S_0".to_string(), str_to_rhs(""));
        assert!(cnf.productions.contains(&prod));
        assert!(cnf.try_find_productions("").is_some());
    }

    #[test]
    fn is_nullable_works() {
        let r = Type2Grammar::new(
            HashSet::from(["S_0".into(), "A".into(), "B".into(), "C".into()]),
            HashSet::from(["a".into(), "b".into(), "c".into()]),
            HashSet::from([
                ("S_0".into(), vec!["A".into(), "b".into(), "B".into()]),
                ("S_0".into(), vec!["C".into()]),
                ("B".into(), vec!["A".into(), "A".into()]),
                ("B".into(), vec!["A".into(), "C".into()]),
                ("C".into(), vec!["b".into()]),
                ("C".into(), vec!["c".into()]),
                ("A".into(), vec!["a".into()]),
                ("A".into(), EPSI.clone()),
            ]),
            "S_0".into(),
        );
        assert!(r.is_ok());
        let g = r.unwrap();
        assert!(g.is_nullable(&"A".into()));
        assert!(g.is_nullable(&"B".into()));
        assert!(!g.is_nullable(&"C".into()));

        let g = g.cnf_del_epsilon();
        assert_eq!(12, g.productions().len());
        assert_eq!(
            g.productions(),
            &HashSet::from_iter(vec![
                ("S_0".into(), str_to_rhs("AbB")),
                ("S_0".into(), str_to_rhs("Ab")),
                ("S_0".into(), str_to_rhs("bB")),
                ("S_0".into(), str_to_rhs("b")),
                ("S_0".into(), str_to_rhs("C")),
                ("B".into(), str_to_rhs("AA")),
                ("B".into(), str_to_rhs("A")),
                ("B".into(), str_to_rhs("AC")),
                ("B".into(), str_to_rhs("C")),
                ("C".into(), str_to_rhs("b")),
                ("C".into(), str_to_rhs("c")),
                ("A".into(), str_to_rhs("a")),
            ])
        );
    }

    #[test]
    fn can_parse_definition_with_cycle() {
        let s = "
    V = { A, B }
    Sigma = { 'a', 'b'  }
    P = { A -> 'B', B -> 'A', A -> 'A', B -> 'B' }
    S = A
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
        let g = r.unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!(cnf.nonterminals(), &HashSet::from_iter(vec!["S_0".into()]));
        assert_eq!(
            cnf.sigma(),
            &HashSet::from_iter(vec!["a".into(), "b".into()])
        );
        assert_eq!(0, cnf.productions().len());
    }

    #[test]
    fn graph_of_unit_rules_works() {
        let s = "
    V = { S, A, B, C, D }
    Sigma = { 'a', 'b'  }
    P = { A -> 'B', B -> 'C', C -> 'D', D -> 'A', A -> 'DD', A -> 'D', D -> 'B' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let graph = g.graph_of_unit_rules();
        assert_eq!(g.nonterminals(), &graph.0);
        assert!(g.is_unit_rule(&("A".into(), vec!["D".into()])));
        assert_eq!(6, graph.1.len());

        let cycles_of_a = Type2Grammar::find_cycles_for(&graph, &"A".to_string());
        assert_eq!(
            HashSet::from_iter(vec![
                vec!["A".into(), "B".into(), "C".into(), "D".into(), "A".into()],
                vec!["A".into(), "D".into(), "A".into()]
            ]),
            cycles_of_a
        );
        let cycles_of_b = Type2Grammar::find_cycles_for(&graph, &"B".to_string());
        assert_eq!(
            HashSet::from_iter(vec![
                vec!["B".into(), "C".into(), "D".into(), "B".into()],
                vec!["B".into(), "C".into(), "D".into(), "A".into(), "B".into()]
            ]),
            cycles_of_b
        );

        let cycles = Type2Grammar::find_cycles(&graph);
        assert_eq!(9, cycles.len());

        let g = g.cnf_del_unit_rules();
        let graph = g.graph_of_unit_rules();
        let cycles = Type2Grammar::find_cycles(&graph);
        assert_eq!(0, cycles.len());
        assert_eq!(1, g.productions().len());
    }

    #[test]
    fn unit_chain_removal_works() {
        let s = "
    V = { S, A, B, C }
    Sigma = { 'a' }
    P = { S -> 'A', A -> 'B', B -> 'C', C -> 'a' }
    S = S
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("a").is_some());
    }

    #[test]
    fn cnf_handles_branching_unit_paths() {
        let s = "
    V = { A, B, X, Y }
    Sigma = { 'a' }
    P = { A -> 'X', B -> 'X', X -> 'Y', Y -> 'a' }
    S = A
    ";
        let g = parser::parse_t2grammar_definition(s).unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert!(cnf.try_find_productions("a").is_some());
        assert!(
            cnf.productions().contains(&("B".into(), vec!["a".into()])),
            "B -> 'a' should exist after CNF because it's not reachable from start and we don't clean these up yet. "
        );
    }

    #[test]
    fn cnf_can_handle_cycles() {
        let s = "
    V = { S, A, B, C }
    Sigma = { 'a', 'b'  }
    P = { A -> 'B', B -> 'C', C -> 'A', S -> 'A', S -> 'B', S -> 'AB', A -> 'aa', B -> 'bb', S -> '' }
    S = S
    ";
        let r = parser::parse_t2grammar_definition(s);
        assert!(r.is_ok());
        let g = r.unwrap();
        let cnf = g.to_cnf();
        assert_cnf(&cnf);
        assert_eq!(
            cnf.sigma(),
            &HashSet::from_iter(vec!["a".into(), "b".into()])
        );
        assert!(cnf.try_find_productions("aa").is_some());
        assert!(cnf.try_find_productions("bb").is_some());
        assert!(cnf.try_find_productions("aabb").is_some());
        let eps_prods = cnf.try_find_productions("");
        assert!(eps_prods.is_some());
        assert_eq!(vec![("S_0".into(), EPSI.clone())], eps_prods.unwrap());
    }
}
