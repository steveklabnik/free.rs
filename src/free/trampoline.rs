pub type Sig<'a, X> = Box<FnOnce() -> X + 'a>;

fn map<'a, X, Y, F:'a>(m: Sig<'a, X>, f: F) -> Sig<'a, Y>
    where
        F: FnOnce(X) -> Y,
{
    box move || f(m.call_once(()))
}

monad!(Trampoline, Sig, map, []);

impl<'a, X:'a> Trampoline<'a, X> {
    #[inline]
    pub fn run(self) -> X {
        self.go(|sbmx: Sig<'a, Box<_>>| *sbmx.call_once(()))
    }
}

#[inline(always)]
pub fn done<'a, X>(a: X) -> Trampoline<'a, X> {
    point(a)
}

#[inline(always)]
pub fn more<'a, X>(ma: Sig<'a, Trampoline<'a, X>>) -> Trampoline<'a, X> {
    wrap(map(ma, |tx| box tx))
}
