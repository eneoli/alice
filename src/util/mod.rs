pub mod counter;

#[macro_export]
macro_rules! s {
    ($x: expr) => {
        String::from($x)
    };
}