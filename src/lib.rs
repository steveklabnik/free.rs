#![crate_name="free"]
#![crate_type="lib"]

#![license = "MIT"]
#![doc(html_root_url = "http://www.rust-ci.org/epsilonz/free.rs/doc/free/")]

#![feature(macro_rules)]
#![feature(overloaded_calls)]
#![feature(unboxed_closures)]

#[macro_export]
macro_rules! free_monad(
    ($Free:ident, $S:ident, $smap:ident, [ $($ctx:ident,)* ]) =>
    {
        pub struct Opaque(*const u8);
        pub type BFnOnce<'a, A, B> = Box<FnOnce<A, B> + 'a>;

        pub enum $Free<'a, $($ctx,)* X> {
            // Pure : a -> Free f a
            Leaf(X),
            // Roll : f (Free f a) -> Free f a
            Nest($S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>),
            // Bind : (() -> Free f b) -> (b -> Free f a) -> Free f a
            Subs(
                BFnOnce<'a, (), $Free<'a, $($ctx,)* Opaque>>,
                BFnOnce<'a, (Opaque,), $Free<'a, $($ctx,)* X>>,
            ),
        }

        impl<'a $(,$ctx:'a)*> $Free<'a, $($ctx,)* Opaque> {
            // NOTE: keep this in sync with bind
            #[inline]
            fn _bind<Y:'a>(
                self,
                f: BFnOnce<'a, (Opaque,), $Free<'a, $($ctx,)* Y>>,
            ) -> $Free<'a, $($ctx,)* Y> {
                match self {
                    Subs(m, g) => {
                        Subs(m, box move |:x|
                            Subs(box move |:|
                                g.call_once((x,)), f))
                    },
                    _ => {
                        Subs(box move |:|
                            self, f)
                    },
                }
            }
        }

        impl<'a $(,$ctx:'a)*, X:'a> $Free<'a, $($ctx,)* X> {
            // NOTE: keep this in sync with _bind
            #[inline]
            pub fn bind<Y:'a, F:'a>(self, f: F) -> $Free<'a, $($ctx,)* Y>
                where
                    F: FnOnce(X) -> $Free<'a, $($ctx,)* Y>,
            {
                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn lhs<'a $(,$ctx:'a)*, X:'a>(
                    m: $Free<'a, $($ctx,)* X>,
                ) -> $Free<'a, $($ctx,)* Opaque> {
                    match m {
                        Leaf(a) => {
                            Leaf(::std::mem::transmute(box a))
                        },
                        Nest(t) => {
                            Nest($smap(t, |:m2: Box<_>|
                                box lhs(*m2)))
                        },
                        Subs(m, f) => {
                            Subs(m, box move |:x|
                                lhs(f.call_once((x,))))
                        },
                    }
                }

                // calls std::mem::transmute
                #[inline(always)]
                unsafe
                fn rhs<'a $(,$ctx:'a)*, X:'a, Y:'a, F:'a>(
                    f: F,
                ) -> BFnOnce<'a, (Opaque,), $Free<'a, $($ctx,)* Y>>
                    where
                        F: FnOnce(X) -> $Free<'a, $($ctx,)* Y>,
                {
                    box move |:ox|
                        f.call_once((*::std::mem::transmute::<_, Box<_>>(ox),))
                }

                // safe because we only coerce (m, f) with compatible types
                unsafe {
                    match self {
                        Subs(m, g) => {
                            Subs(m, box move |:x|
                                Subs(box move |:|
                                    lhs(g.call_once((x,))), rhs(f)))
                        },
                        _ => {
                            Subs(box move |:|
                                lhs(self), rhs(f))
                        },
                    }
                }
            }

            #[inline]
            fn resume(
                mut self,
            ) -> Result<X, $S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>> {
                loop { match self {
                    Leaf(a) => {
                        return Ok (a)
                    },
                    Nest(t) => {
                        return Err(t)
                    },
                    Subs(ma, f) => {
                        match ma.call_once(()) {
                            Leaf(a) => {
                                self = f.call_once((a,))
                            },
                            Nest(t) => {
                                return Err($smap(t,
                                    move |:m:Box<$Free<'a, $($ctx,)* _>>|
                                        box m._bind(f)))
                            },
                            Subs(mb, g) => {
                                self = mb
                                    .call_once(())
                                    ._bind(box move |:pb| g
                                        .call_once((pb,))
                                        ._bind(f))
                            },
                        }
                    },
                }}
            }

            #[inline]
            fn go<F>(mut self, f: F) -> X
                where
                    // f must be a Fn since we may call it many times
                    F: Fn($S<'a, $($ctx,)* Box<$Free<'a, $($ctx,)* X>>>)
                        -> $Free<'a, $($ctx,)* X>,
            {
                loop { match self.resume() {
                    Ok (a) => {
                        return a
                    },
                    Err(t) => {
                        self = f.call((t,))
                    },
                }}
            }
        }

    };
)
