use super::{
    Abs,
    FunOnce,
};

pub type TakeA<'a, A, X> = Box<FnOnce<(A,), X> + 'a>; // A should be lazy
pub fn map<'a, A, X, Y, F:'a>(m: TakeA<'a, A, X>, f: F) -> TakeA<'a, A, Y>
    where
        F: FnOnce(X) -> Y,
{
    box move |:a| f(m.call_once((a,)))
}
free!(Sink, TakeA, map, [ A, ])
