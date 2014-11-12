#![crate_name="free"]
#![crate_type="lib"]

#![license = "MIT"]
#![doc(html_root_url = "http://www.rust-ci.org/epsilonz/free.rs/doc/free/")]

#![feature(macro_rules)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]

#[macro_export]
macro_rules! free_monad(
    ($Ty:ident, $S:ident, $smap:ident, [ $($ctx:ident,)* ]) =>
    {
        struct Abs(*const u8);
        type BFnOnce<'a, A, B> = Box<FnOnce<A, B> + 'a>;

        #[allow(dead_code)]
        enum Free<'a, $($ctx,)* X> {
            Pure(X),
            Roll($S<'a, $($ctx,)* Box<Free<'a, $($ctx,)* X>>>),
            Subs(
                BFnOnce<'a, (), Free<'a, $($ctx,)* Abs>>,
                BFnOnce<'a, (Abs,), Free<'a, $($ctx,)* X>>,
            ),
        }

        impl<'a $(,$ctx:'a)*> Free<'a, $($ctx,)* Abs> {
            #[allow(dead_code)]
            #[inline]
            fn _map<Y:'a, F:'a>(self, f: BFnOnce<'a, (Abs,), Y>,) -> Free<'a, $($ctx,)* Y> {
                self._bind(box move |:x| Pure(f.call_once((x,))))
            }

            // NOTE: keep this in sync with bind
            #[inline]
            fn _bind<Y:'a>(self, f: BFnOnce<'a, (Abs,), Free<'a, $($ctx,)* Y>>) -> Free<'a, $($ctx,)* Y> {
                match self {
                    Subs(m, g) => Subs(m, box move |:x| Subs(box move |:| g.call_once((x,)), f)),
                    _ => Subs(box move |:| { self }, f),
                }
            }
        }

        impl<'a $(,$ctx:'a)*, X:'a> Free<'a, $($ctx,)* X> {
            #[inline(always)]
            fn wrap(self) -> $Ty<'a, $($ctx,)* X> {
                $Ty(self)
            }

            #[inline]
            fn map<Y:'a, F:'a>(self, f: F) -> Free<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> Y,
            {
                self.bind(move |:x| Pure(f(x)))
            }

            // NOTE: keep this in sync with _bind
            #[inline]
            fn bind<Y:'a, F:'a>(self, f: F) -> Free<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> Free<'a, $($ctx,)* Y>,
            {
                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn lhs<'a $(,$ctx:'a)*, X:'a>(m: Free<'a, $($ctx,)* X>) -> Free<'a, $($ctx,)* Abs> {
                    m.map(|:x| ::std::mem::transmute(box x))
                }

                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn rhs<'a $(,$ctx:'a)*, X:'a, Y:'a, F:'a>(f: F) -> BFnOnce<'a, (Abs,), Free<'a, $($ctx,)* Y>>
                    where
                        F: FnOnce(X) -> Free<'a, $($ctx,)* Y>,
                {
                    box move |:ox| f.call_once((*::std::mem::transmute::<_, Box<_>>(ox),))
                }

                // safe because we only coerce (m, f) with compatible types
                unsafe {
                    match self {
                        Subs(m, g) => Subs(m, box move |:x| Subs(box move |:| lhs(g.call_once((x,))), rhs(f))),
                        _ => Subs(box move |:| lhs(self), rhs(f)),
                    }
                }
            }

            #[inline]
            fn fold<Y, P, R>(self, p: P, r: R) -> Y
                where
                    P: Fn(X) -> Y,
                    R: Fn($S<'a, $($ctx,)* Box<Free<'a, $($ctx,)* X>>>) -> Y,
            {
                match self.resume() {
                    Ok (a) => p.call_once((a,)),
                    Err(t) => r.call_once((t,)),
                }
            }

            #[inline]
            fn resume(self) -> Result<X, $S<'a, $($ctx,)* Box<Free<'a, $($ctx,)* X>>>> {
                match self {
                    Pure(a) => Ok (a),
                    Roll(t) => Err(t),
                    Subs(ma, f) => {
                        match ma.call_once(()) {
                            Pure(a) => f.call_once((a,)).resume(),
                            Roll(t) => Err($smap(t, move |:m:Box<Free<'a, _>> | box m._bind(f))),
                            Subs(mb, g) => mb.call_once(())._bind(box move |:pb| g.call_once((pb,))._bind(f)).resume(),
                        }
                    },
                }
            }

            #[inline]
            fn bounce<F>(self, f: F) -> Free<'a, $($ctx,)* X>
                where
                    F: FnOnce($S<'a, $($ctx,)* Box<Free<'a, $($ctx,)* X>>>) -> Free<'a, $($ctx,)* X>,
            {
                match self.resume() {
                    Ok (a) => Pure(a),
                    Err(t) => f.call_once((t,)),
                }
            }

            #[allow(dead_code)]
            #[inline]
            fn go<F>(mut self, f: F) -> X
                where
                    // f must be a Fn since we may call it many times
                    F: Fn($S<'a, $($ctx,)* Box<Free<'a, $($ctx,)* X>>>) -> Free<'a, $($ctx,)* X>,
            {
                let acc: X;
                loop { match self.resume() {
                    Ok (a) => { acc  = a; break     },
                    Err(t) => { self = f.call((t,)) },
                }};
                acc
            }
        }

        pub struct $Ty<'a $(,$ctx:'a)*, X:'a>(Free<'a, $($ctx,)* X>);

        impl<'a $(,$ctx:'a)*, X:'a> $Ty<'a, $($ctx,)* X> {
            #[inline(always)]
            fn proj(self) -> Free<'a, $($ctx,)* X> {
                let $Ty(mx) = self; mx
            }

            #[inline]
            pub fn map<Y:'a, F:'a>(self, f: F) -> $Ty<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> Y,
            {
                self.proj().map(f).wrap()
            }

            // NOTE: keep this in sync with _bind
            #[inline]
            pub fn bind<Y:'a, F:'a>(self, f: F) -> $Ty<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> $Ty<'a, $($ctx,)* Y>,
            {
                self.proj()
                    .bind(move |:x| f(x).proj())
                    .wrap()
            }

            #[inline]
            pub fn fold<Y, P, R>(self, p: P, r: R) -> Y
                where
                    P: Fn(X) -> Y,
                    R: Fn($S<'a, $($ctx,)* Box<$Ty<'a, $($ctx,)* X>>>) -> Y,
            {
                self.proj()
                    .fold(p, |&:sbmx:$S<'a, _>| {
                        r($smap(sbmx, |:bmx:Box<Free<'a, _>>| {
                            box bmx.wrap()
                        }))
                    })
            }

            #[inline]
            pub fn resume(self) -> Result<X, $S<'a, $($ctx,)* Box<$Ty<'a, $($ctx,)* X>>>> {
                self.proj()
                    .resume()
                    .map_err(|sbmx| {
                        $smap(sbmx, |:bmx:Box<Free<'a, _>>| {
                            box bmx.wrap()
                        })
                    })
            }

            #[inline]
            pub fn bounce<F>(self, f: F) -> $Ty<'a, $($ctx,)* X>
                where
                    F: FnOnce($S<'a, $($ctx,)* Box<$Ty<'a, $($ctx,)* X>>>) -> $Ty<'a, $($ctx,)* X>,
            {
                self.proj()
                    .bounce(move |:sbmx| {
                        f($smap(sbmx, |:bmx:Box<Free<'a, _>>| {
                            box bmx.wrap()
                        })).proj()
                    }).wrap()
            }

            #[allow(dead_code)]
            #[inline]
            pub fn go<F>(self, f: F) -> X
                where
                    // f must be a Fn since we may call it many times
                    F: Fn($S<'a, $($ctx,)* Box<$Ty<'a, $($ctx,)* X>>>) -> $Ty<'a, $($ctx,)* X>,
            {
                self.proj()
                    .go(|&:sbmx| {
                        f($smap(sbmx, |:bmx:Box<Free<'a, _>>| {
                            box bmx.wrap()
                        })).proj()
                    })
            }
        }

    };
)
