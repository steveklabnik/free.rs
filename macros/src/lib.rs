#![crate_name="free_macros"]
#![crate_type="lib"]

#![license = "MIT"]
#![doc(html_root_url = "http://www.rust-ci.org/epsilonz/free.rs/doc/free/")]

#![feature(macro_rules)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]

pub struct Abs(*const u8);
pub type FunOnce<'a, A, B> = Box<FnOnce<A, B> + 'a>;

#[doc(hidden)]
#[macro_export]
macro_rules! __free_impl(
    ($Free:ident, $S:ident, $smap:ident, [ $($ctx:ident,)* ]) =>
    {
        pub enum $Free<'a, $($ctx,)* X> {
            Pure(X),
            Roll($S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>),
            Subs(
                FunOnce<'a,     (), $Free<'a, $($ctx,)* Abs>>,
                FunOnce<'a, (Abs,), $Free<'a, $($ctx,)*   X>>,
            ),
        }

        impl<'a $(,$ctx:'a)*> $Free<'a, $($ctx,)* Abs> {
            #[allow(dead_code)]
            #[inline]
            fn map_abs<Y:'a, F:'a>(
                self,
                f: FunOnce<'a, (Abs,),   Y>,
            ) ->     $Free<'a, $($ctx,)* Y>
            {
                self.bind_abs(box move |:x| Pure(f.call_once((x,))))
            }

            // NOTE: keep this in sync with bind
            #[inline]
            fn bind_abs<Y:'a>(
                self,
                f: FunOnce<'a, (Abs,),
                    $Free<'a, $($ctx,)* Y>>,
            ) ->    $Free<'a, $($ctx,)* Y>
            {
                match self {
                    Subs(m, g) => Subs(m, box move |:x| {
                        Subs(box move |:| {
                            g.call_once((x,))
                        }, f)
                    }),
                    _          => Subs(box move |:| { self }, f),
                }
            }
        }

        impl<'a $(,$ctx:'a)*, X:'a> $Free<'a, $($ctx,)* X> {
            #[inline]
            pub fn map<Y:'a, F:'a>(self, f: F)
                                 -> $Free<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> Y,
            {
                self.bind(move |:x| Pure(f(x)))
            }

            // NOTE: keep this in sync with bind_abs
            #[inline]
            pub fn bind<Y:'a, F:'a>(self, f: F)
                                 -> $Free<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> $Free<'a, $($ctx,)* Y>,
            {
                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn coe_lhs<'a $(,$ctx:'a)*, X:'a>(m: $Free<'a, $($ctx,)* X  >)
                                                  -> $Free<'a, $($ctx,)* Abs>
                {
                    m.map(|:x| ::std::mem::transmute(box x))
                }

                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn coe_rhs<'a $(,$ctx:'a)*, X:'a, Y:'a, F:'a>(
                    f: F
                ) -> FunOnce<'a, (Abs,), $Free<'a, $($ctx,)* Y>>
                    where
                        F: FnOnce(X) ->  $Free<'a, $($ctx,)* Y>,
                {
                    box move |:ox| {
                        let box x: Box<X> = ::std::mem::transmute(ox);
                        f.call_once((x,))
                    }
                }

                // safe because we only coerce (m, f) with compatible types
                unsafe {
                    match self {
                        Subs(m, g) =>
                            Subs(m, box move |:x| {
                                Subs(box move |:| {
                                    coe_lhs(g.call_once((x,)))
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
            pub fn fold<Y, P, R>(self, p: P, r: R) -> Y
                where
                    P: Fn(X) -> Y,
                    R: Fn($S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>) -> Y,
            {
                match self.resume() {
                    Ok (a) => p.call_once((a,)),
                    Err(t) => r.call_once((t,)),
                }
            }

            #[inline]
            pub fn resume(self) ->
                Result
                    <
                        X,
                        $S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>
                    >
                {
                match self {
                    Pure(a) => Ok (a),
                    Roll(t) => Err(t),
                    Subs(ma, f) => {
                        match ma.call_once(()) {
                            Pure(a) => f.call_once((a,)).resume(),
                            Roll(t) => Err({
                                $smap(t, move |:m: Box<$Free<'a, $($ctx,)* Abs>>| {
                                    box m.bind_abs(f)
                                })
                            }),
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
            pub fn bounce<F>(self, f: F) ->    $Free<'a, $($ctx,)* X>
                where
                    F: FnOnce($S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>)
                                               ->  $Free<'a, $($ctx,)* X>,
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
                    F: Fn($S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>)
                                           ->  $Free<'a, $($ctx,)* X>,
            {
                let acc: X;
                loop { match self.resume() {
                    Ok (a) => { acc  = a; break     },
                    Err(t) => { self = f.call((t,)) },
                }};
                acc
            }
        }
    };
)

#[macro_export]
macro_rules! free(
    ($Free:ident, $S:ident, $smap:ident, [ $($ctx:ident,)* ]) =>
    {
        __free_impl!($Free, $S, $smap, [ $($ctx,)* ])
    };
)
