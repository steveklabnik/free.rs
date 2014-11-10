macro_rules! free(
    ($Free:ident, $S:ident) =>
    {
        pub type Abs               = *const u8;
        pub type FunOnce<'a, A, B> = Box<FnOnce<A, B> + 'a>;

        pub enum $Free<'a, A> {
            Pure(A),
            Roll($S<'a, $Free<'a, A>>),
            Subs(
                FunOnce<'a,     (), $Free<'a, Abs>>,
                FunOnce<'a, (Abs,), $Free<'a, A  >>,
            ),
        }

        impl<'a> $Free<'a, Abs> {
            #[inline]
            fn map_abs<B, F:'a>(
                self,
                f: FunOnce<'a, (Abs,), B>,
            ) -> $Free<'a, B>
            {
                self.bind_abs(box move |:a| Pure(f.call_once((a,))))
            }

            // NOTE: keep this in sync with bind
            #[inline]
            fn bind_abs<B>(
                self,
                f: FunOnce<'a, (Abs,), $Free<'a, B>>,
            ) -> $Free<'a, B>
            {
                match self {
                    Subs(m, g) => Subs(m, box move |:a| {
                        Subs(box move |:| {
                            g.call_once((a,))
                        }, f)
                    }),
                    _          => Subs(box move |:| { self }, f),
                }
            }
        }

        impl<'a, A:'a> $Free<'a, A> {
            #[inline]
            pub fn map<B, F:'a>(self, f: F) -> $Free<'a, B>
                where
                    F: FnOnce(A) -> B,
            {
                self.bind(move |:a| Pure(f.call_once((a,))))
            }

            // NOTE: keep this in sync with bind_abs
            #[inline]
            pub fn bind<B, F:'a>(self, f: F) -> $Free<'a, B>
                where
                    F: FnOnce(A) -> $Free<'a, B>,
            {
                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn coe_lhs<'a, A>(m: $Free<'a, A>) -> $Free<'a, Abs>
                {
                    match m {
                        Pure(a)   => Pure(::std::mem::transmute(box a)),
                        Roll(t)   => Roll(   box move |: | coe_lhs(t.call_once(  ()))),
                        Subs(m,f) => Subs(m, box move |:x| coe_lhs(f.call_once((x,)))),
                    }
                }

                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn coe_rhs<'a, A, B, F:'a>(
                    f: F
                ) -> FunOnce<'a, (Abs,), $Free<'a, B>>
                    where
                        F: FnOnce(A)   -> $Free<'a, B>,
                {
                    box move |:oa| {
                        let box a: Box<A> = ::std::mem::transmute(oa);
                        f.call_once((a,))
                    }
                }

                // safe because we only ever coerce (m, f) with compatible types
                unsafe {
                    match self {
                        Subs(m, g) =>
                            Subs(m, box move |:a| {
                                Subs(box move |:| {
                                    coe_lhs(g.call_once((a,)))
                                }, coe_rhs(f))
                            }),
                        _          =>
                            Subs(
                                box move |:| { coe_lhs(self) },
                                coe_rhs(f)
                            ),
                    }
                }
            }

            #[inline]
            pub fn fold<B, P, R>(self, p: P, r: R) -> B
                where
                    P: Fn(A) -> B,
                    R: Fn($S<'a, $Free<'a, A>>) -> B,
            {
                match self.resume() {
                    Ok (a) => p.call_once((a,)),
                    Err(t) => r.call_once((t,)),
                }
            }

            #[inline]
            pub fn resume(self) -> Result<A, $S<'a, $Free<'a, A>>> {
                match self {
                    Pure(a) => Ok (a),
                    Roll(t) => Err(t),
                    Subs(ma, f) => {
                        match ma.call_once(()) {
                            Pure(a) => f.call_once((a,)).resume(),
                            Roll(t) => {
                                Err({
                                    // FIXME: Without the annotation, rustc thinks we
                                    // need a Send bound. Maybe report upstream?
                                    let t: $S<'a, $Free<'a, A>> = box move |:| {
                                        Subs(t, f)
                                    };
                                    t
                                })
                            },
                            Subs(mb, g) => {
                                mb.call_once(   ()).bind_abs(box move |:pb| {
                                    g.call_once((pb,)).bind_abs(f)
                                }).resume()
                            },
                        }
                    },
                }
            }

            #[inline]
            pub fn bounce<F>(self, f: F) -> $Free<'a, A>
                where
                    F: Fn($S<'a, $Free<'a, A>>) -> $Free<'a, A>,
            {
                match self.resume() {
                    Ok (a) => Pure(a),
                    Err(t) => f.call((t,)),
                }
            }

            #[inline]
            fn go<F>(mut self, f: F) -> A
                where
                    F: Fn($S<'a, $Free<'a, A>>) -> $Free<'a, A>,
            {
                let acc: A;
                loop { match self.resume() {
                    Ok (a) => { acc  = a; break     },
                    Err(t) => { self = f.call((t,)) },
                }};
                acc
            }
        }

    };
)

pub type Lazy<'a, X> = Box<FnOnce<(), X> + 'a>;
free!(Trampoline, Lazy)

// FIXME: need to generalize this to work with types like λ A. ∀ X. (A, X).
// I think we should be able to do this by passing around a context of types.
