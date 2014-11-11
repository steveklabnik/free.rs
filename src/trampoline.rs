use free_macros::{
    Abs,
    FunOnce,
};

pub type Lazy<'a, X> = Box<FnOnce<(), X> + 'a>;
pub fn map<'a, X, Y, F:'a>(m: Lazy<'a, X>, f: F) -> Lazy<'a, Y>
    where
        F: FnOnce(X) -> Y,
{
    box move |:| f(m.call_once(()))
}
free!(Trampoline, Lazy, map, [])
