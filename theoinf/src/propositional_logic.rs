use std::collections::HashMap;

pub enum Expr<'a> {
    Val { name: &'a str },
    Not { x: &'a Expr<'a> },
    Or { x: &'a Expr<'a>, y: &'a Expr<'a> },
    And { x: &'a Expr<'a>, y: &'a Expr<'a> },
    True,
    False,
}

pub fn eval(assignment: &HashMap<&str, bool>, expr: &Expr) -> bool {
    match expr {
        Expr::Val { name: x } => assignment[x],
        Expr::Not { x } => !eval(assignment, x),
        Expr::Or { x, y } => eval(assignment, x) || eval(assignment, y),
        Expr::And { x, y } => eval(assignment, x) && eval(assignment, y),
        Expr::True => true,
        Expr::False => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_works() {
        let a = Expr::Val { name: "a" };
        let b = Expr::Val { name: "b" };
        let mut assignment = HashMap::new();
        assignment.insert("a", false);
        assignment.insert("b", true);
        assert!(!eval(&assignment, &a));
        assert!(eval(&assignment, &b))
    }

    #[test]
    fn not_works() {
        let assignment = HashMap::new();
        let not_true = Expr::Not { x: &Expr::True };
        let not_false = Expr::Not { x: &Expr::False };
        assert!(!eval(&assignment, &not_true));
        assert!(eval(&assignment, &not_false))
    }

    #[test]
    fn or_works() {
        let assignment = HashMap::new();

        let expr = Expr::Or {
            x: &Expr::False,
            y: &Expr::False,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::Or {
            x: &Expr::False,
            y: &Expr::True,
        };
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or {
            x: &Expr::True,
            y: &Expr::False,
        };
        assert!(eval(&assignment, &expr));

        let expr = Expr::Or {
            x: &Expr::True,
            y: &Expr::True,
        };
        assert!(eval(&assignment, &expr));
    }

    #[test]
    fn and_works() {
        let assignment = HashMap::new();

        let expr = Expr::And {
            x: &Expr::False,
            y: &Expr::False,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: &Expr::False,
            y: &Expr::True,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: &Expr::True,
            y: &Expr::False,
        };
        assert!(!eval(&assignment, &expr));

        let expr = Expr::And {
            x: &Expr::True,
            y: &Expr::True,
        };
        assert!(eval(&assignment, &expr));
    }
}
