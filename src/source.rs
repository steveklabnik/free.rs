use super::{
    Abs,
    FunOnce,
};

pub type WithA<'a, A, X> = (A, X);
pub fn map<'a, A, X, Y, F>(m: WithA<'a, A, X>, f: F) -> WithA<'a, A, Y>
    where
        F: FnOnce(X) -> Y,
{
    match m {
        (a, x) => (a, f(x))
    }
}
free!(Source, WithA, map, [ A, ])
