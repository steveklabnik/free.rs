pub type Lazy<'a, X> = Box<FnOnce<(), X> + 'a>;

pub fn map<'a, X, Y, F:'a>(m: Lazy<'a, X>, f: F) -> Lazy<'a, Y>
    where
        F: FnOnce(X) -> Y,
{
    box move |:| f(m.call_once(()))
}

free_monad!(Trampoline, Lazy, map, [])

impl<'a, X:'a> Trampoline<'a, X> {
    #[inline]
    pub fn run(self) -> X {
        self.go(|&:sbmx: Lazy<'a, Box<_>>| *sbmx.call_once(()))
    }
}

#[inline(always)]
pub fn done<'a, X>(a: X) -> Trampoline<'a, X> {
    Leaf(a)
}

#[inline(always)]
pub fn more<'a, X>(ma:Lazy<'a, Trampoline<'a, X>>) -> Trampoline<'a, X> {
    Nest(map(ma, |:tx: Trampoline<'a, _>| box tx))
}
