use theoinf::propositional_logic::{print_truth_table, truth_table};
fn main() {
    let formula = "a <=> b";
    println!("{formula}");
    let tt = truth_table(formula);
    print_truth_table(&tt.unwrap());
}
