#[macro_export]
macro_rules! monad(
    ($M:ident, $Sig:ident, $sig_map:ident, [ $($ctx:ident,)* ]) =>
    {

        #[allow(dead_code)]
        struct Opaque(*const u8);

        // Leaf ~ Pure : a -> _M f a
        // Nest ~ Roll : f (_M f a) -> _M f a
        // Subs ~ Bind : (() -> _M f b) -> (b -> _M f a) -> _M f a

        enum _M<'a, $($ctx,)* X> {
            Leaf(X),
            Nest($Sig<'a, $($ctx,)* Box<_M<'a, $($ctx,)* X>>>),
            Subs( // Coyoneda f a ~ forall i. (f i, i -> a)
                Box<FnOnce() -> _M<'a, $($ctx,)* Opaque> + 'a>,
                Box<FnOnce(Opaque,) -> _M<'a, $($ctx,)* X> + 'a>,
            ),
        }

        impl<'a $(,$ctx:'a)*> _M<'a, $($ctx,)* Opaque> {
            // NOTE: keep this in sync with bind
            #[allow(dead_code)]
            #[inline]
            fn _bind<Y:'a>(
                self,
                f: Box<FnOnce(Opaque,) -> _M<'a, $($ctx,)* Y> + 'a>,
            ) -> _M<'a, $($ctx,)* Y> {
                match self {
                    _M::Subs(m, g) => {
                        _M::Subs(m, box move |x|
                            _M::Subs(box move ||
                                g.call_once((x,)), f))
                    },
                    _ => {
                        _M::Subs(box move ||
                            self, f)
                    },
                }
            }
        }

        impl<'a $(,$ctx:'a)*, X:'a> _M<'a, $($ctx,)* X> {
            // NOTE: keep this in sync with _bind
            #[allow(dead_code)]
            #[inline]
            fn bind<Y:'a, F:'a>(self, f: F) -> _M<'a, $($ctx,)* Y>
                where
                F: FnOnce(X) -> _M<'a, $($ctx,)* Y>,
            {
                // calls std::mem::transmute
                #[allow(dead_code)]
                #[inline]
                unsafe
                fn lhs<'a $(,$ctx:'a)*, X:'a>(
                    m: _M<'a, $($ctx,)* X>,
                ) -> _M<'a, $($ctx,)* Opaque> {
                    match m {
                        _M::Leaf(a) => {
                            _M::Leaf(::std::mem::transmute(box a))
                        },
                        _M::Nest(t) => {
                            _M::Nest($sig_map(t, |m2: Box<_>|
                                box lhs(*m2)))
                        },
                        _M::Subs(m, f) => {
                            _M::Subs(m, box move |x|
                                lhs(f.call_once((x,))))
                        },
                    }
                }

                // calls std::mem::transmute
                #[allow(dead_code)]
                #[inline]
                unsafe
                fn rhs<'a $(,$ctx:'a)*, X:'a, Y:'a, F:'a>(
                    f: F,
                ) -> Box<FnOnce<(Opaque,), _M<'a, $($ctx,)* Y>> + 'a>
                    where
                    F: FnOnce(X) -> _M<'a, $($ctx,)* Y>,
                {
                    box move |ox|
                        f.call_once((*::std::mem::transmute::<_, Box<_>>(ox),))
                }

                // safe because we only coerce (m, f) with compatible types
                match self {
                    _M::Subs(m, g) => {
                        _M::Subs(m, box move |x| unsafe {
                            _M::Subs(box move ||
                                lhs(g.call_once((x,))), rhs(f))
                        })
                    },
                    _ => { unsafe {
                        _M::Subs(box move ||
                            lhs(self), rhs(f))
                    }},
                }
            }

            #[allow(dead_code)]
            #[inline]
            fn resume(
                mut self,
            ) -> Result<X, $Sig<'a, $($ctx,)* Box<_M<'a, $($ctx,)* X>>>> {
                loop { match self {
                    _M::Leaf(a) => {
                        return Ok(a)
                    },
                    _M::Nest(t) => {
                        return Err(t)
                    },
                    _M::Subs(ma, f) => {
                        match ma.call_once(()) {
                            _M::Leaf(a) => {
                                self = f.call_once((a,))
                            },
                            _M::Nest(t) => {
                                return Err($sig_map(t,
                                    move |m: Box<_M<'a, $($ctx,)* _>>|
                                        box m._bind(f)))
                            },
                            _M::Subs(mb, g) => {
                                self = mb
                                    .call_once(())
                                    ._bind(box move |pb| g
                                        .call_once((pb,))
                                        ._bind(f))
                            },
                        }
                    },
                }}
            }

            #[allow(dead_code)]
            #[inline]
            fn go<F>(mut self, f: F) -> X
                where
                // f must be a Fn since we may call it many times
                F: Fn($Sig<'a, $($ctx,)* Box<_M<'a, $($ctx,)* X>>>)
                    -> _M<'a, $($ctx,)* X>,
            {
                loop { match self.resume() {
                    Ok(a) => {
                        return a
                    },
                    Err(t) => {
                        self = f.call((t,))
                    },
                }}
            }

        }

        #[allow(dead_code)]
        pub struct $M<'a $(,$ctx:'a)*, X:'a>(_M<'a, $($ctx,)* X>);

        impl<'a $(,$ctx:'a)*, X:'a> $M<'a, $($ctx,)* X> {
            #[allow(dead_code)]
            #[inline]
            pub fn map<Y:'a, F:'a>(self, f: F) -> $M<'a, $($ctx,)* Y>
                where
                F: FnOnce(X) -> Y,
            {
                let $M(m) = self;
                $M(m.bind(move |x| _M::Leaf(f(x))))
            }

            #[allow(dead_code)]
            #[inline]
            pub fn point(a: X) -> $M<'a, $($ctx,)* X> {
                $M(_M::Leaf(a))
            }

            // NOTE: keep this in sync with _bind
            #[allow(dead_code)]
            #[inline]
            pub fn bind<Y:'a, F:'a>(self, f: F) -> $M<'a, $($ctx,)* Y>
                where
                F: FnOnce(X) -> $M<'a, $($ctx,)* Y>,
            {
                let $M(m0) = self;
                $M(m0.bind(move |x| {
                    let $M(m1) = f(x);
                    m1
                }))
            }

            #[allow(dead_code)]
            #[inline]
            pub fn seq<Y:'a>(self, m: $M<'a, $($ctx,)* Y>) -> $M<'a, $($ctx,)* Y> {
                self.bind(move |_| m)
            }

            #[allow(dead_code)]
            #[inline]
            pub fn resume(self) -> Result<X, $Sig<'a, $($ctx,)* Box<$M<'a, $($ctx,)* X>>>> {
                let $M(m0) = self;
                match m0.resume() {
                    Ok(a) => {
                        Ok(a)
                    },
                    Err(sbmx) => {
                        Err($sig_map(sbmx, |bmx: Box<_>| box $M(*bmx)))
                    },
                }
            }

            #[allow(dead_code)]
            #[inline]
            pub fn go<F>(self, f: F) -> X
                where
                // f must be a Fn since we may call it many times
                F: Fn($Sig<'a, $($ctx,)* Box<$M<'a, $($ctx,)* X>>>) -> $M<'a, $($ctx,)* X>,
            {
                let $M(m0) = self;
                m0.go(|sbmx| {
                    let $M(m1) = f($sig_map(sbmx, |bmx: Box<_>| box $M(*bmx)));
                    m1
                })
            }

        }

        #[allow(dead_code)]
        #[inline]
        pub fn point<'a $(,$ctx:'a)*, X:'a>(a: X) -> $M<'a, $($ctx,)* X> {
            $M::point(a)
        }

        #[allow(dead_code)]
        #[inline]
        pub fn bind<'a $(,$ctx:'a)*, X:'a, Y:'a, F:'a>(
            m: $M<'a, $($ctx,)* X>,
            f: F,
        ) -> $M<'a, $($ctx,)* Y>
            where
            F: FnOnce(X) -> $M<'a, $($ctx,)* Y>,
        {
            m.bind(f)
        }

        #[allow(dead_code)]
        #[inline]
        pub fn wrap<'a $(,$ctx:'a)*, X:'a>(
            sbmx: $Sig<'a, $($ctx,)* Box<$M<'a, $($ctx,)* X>>>
        ) -> $M<'a, $($ctx,)* X> {
            $M(_M::Nest($sig_map(sbmx, |bmx| {
                let box $M(m) = bmx;
                box m
            })))
        }

    };
);
