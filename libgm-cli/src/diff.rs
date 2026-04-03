use similar_asserts::SimpleDiff;
use std::fmt::Debug;

pub fn print_diff<T: Debug + PartialEq + ?Sized>(old: &T, new: &T) {
    let old = format!("{old:#?}");
    let new = format!("{new:#?}");
    println!("{}", SimpleDiff::from_str(&old, &new, "Old", "New"));
}
