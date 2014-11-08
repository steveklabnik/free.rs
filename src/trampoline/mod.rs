#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::mem::{
    transmute,
};

// Private API

type F<'a,A> = proc():'a -> A;

enum Free<'a,A> {
    Pure(A),
    Roll(F<'a,Free<'a,A>>),
    Bind(Box<Free<'a,*const u8>>, proc(*const u8):'a -> Free<'a,A>),
}

impl<'a,A> Free<'a,A> {
    #[inline]
    fn resume(self) -> Result<A,F<'a,Free<'a,A>>> {
        match self {
            Pure(val) => Ok (val),
            Roll(thk) => Err(thk),
            Bind(box mon2, con2) => {
                match mon2 {
                    Pure(val) => con2(val).resume(),
                    Roll(thk1) => {
                        Err({
                            // FIXME: Without the annotation, rustc thinks we
                            // need a Send bound. Maybe report upstream?
                            let thk2: F<'a,Free<'a,A>> = proc() {
                                Bind(box thk1(), con2)
                            };
                            thk2
                        })
                    },
                    Bind(bmon1, con1) => {
                        Bind(bmon1, proc(ptr1)
                            Bind(box con1(ptr1), con2))
                                .resume()
                    },
                }
            },
        }
    }

    #[inline]
    fn go(mut self, f:|F<Free<A>>| -> Free<A>) -> A {
        let res: A;
        loop { match self.resume() {
            Ok (val) => { res  = val; break },
            Err(cmp) => { self = f(cmp)     },
        }};
        res
    }

    #[inline]
    fn run(self) -> A {
        self.go(force)
    }
}

#[inline(always)]
fn coe_m<'a,A>(t:Trampoline<'a,A>) -> Free<'a,*const u8> {
    let Trampoline(m) = t;
    match m {
        Pure(a) => Pure(unsafe { transmute(box a) }),
        Roll(k) => Roll(proc() coe_m(Trampoline(k()))),
        Bind(m,f) => Bind(m, proc(x) coe_m(Trampoline(f(x)))),
    }
}

#[inline(always)]
fn coe_f<'a,A,B>(k:proc(A):'a -> Trampoline<'a,B>) -> proc(*const u8):'a -> Free<'a,B> {
    proc(x) {
        let box a: Box<A> = unsafe { transmute(x) };
        let Trampoline(mb) = k(a);
        mb
    }
}

// Public API

pub type Thunk<'a,A> = proc():'a -> A;

#[inline(always)]
pub fn force<'a,A>(k:Thunk<'a,A>) -> A {
    k()
}

pub struct Trampoline<'a,A>(Free<'a,A>);

impl<'a,A> Trampoline<'a,A> {
    #[inline(always)]
    pub fn bind<A,B>(self, f:proc(A):'a -> Trampoline<'a,B>) -> Trampoline<'a,B> {
        Trampoline(Bind(box coe_m(self), coe_f(f)))
    }

    #[inline]
    pub fn run(self) -> A {
        let Trampoline(m) = self;
        m.go(force)
    }
}

#[inline(always)]
pub fn done<'a,A>(a:A) -> Trampoline<'a,A> {
    Trampoline(Pure(a))
}

#[inline(always)]
pub fn more<'a,A>(k:Thunk<'a,Trampoline<'a,A>>) -> Trampoline<'a,A> {
    Trampoline(Roll(proc() {
        let Trampoline(ma) = k();
        ma
    }))
}

#[cfg(test)]
mod tests {
    extern crate num;

    use self::num::BigInt;
    use super::{
        Trampoline,
        done,
        more,
    };

    fn factorial<'a>(n:uint, acc:BigInt) -> Trampoline<'a,BigInt> {
        if n <= 2 {
            done(acc)
        } else {
            let nb: BigInt = FromPrimitive::from_uint(n).unwrap();
            more(proc() {
                factorial(n - 1, nb * acc)
            })
        }
    }

    #[test]
    fn welp() {
        let acc: BigInt = FromPrimitive::from_uint(1u).unwrap();
        println!("{}", factorial(1500, acc).run())
    }
}
