#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// FIXME: make a macro for this

pub type F<'a,A> = proc():'a -> A;

#[inline]
pub fn force<'a,A>(f:F<'a,A>) -> A {
    f()
}

enum Free<'a,A> {
    Pure(A),
    Roll(F<'a,Free<'a,A>>),
}

impl<'a,A> Free<'a,A> {
    #[inline]
    pub fn resume(self) -> Result<A,F<'a,Free<'a,A>>> {
        match self {
            Pure(val) => { Ok (val) },
            Roll(cmp) => { Err(cmp) },
        }
    }

    #[inline]
    pub fn go(mut self, f:|F<'a,Free<'a,A>>| -> Free<'a,A>) -> A {
        let res: A;
        loop { match self.resume() {
            Ok (val) => { res  = val; break },
            Err(cmp) => { self = f(cmp)     },
        }};
        res
    }
}

#[cfg(test)]
mod tests {
    extern crate num;

    use self::num::BigInt;
    use super::{
        Free,
        Pure,
        Roll,
        force,
    };

    fn factorial<'a>(n:uint, acc:BigInt) -> Free<'a,BigInt> {
        if n <= 2 {
            Pure(acc)
        } else {
            let nb: BigInt = FromPrimitive::from_uint(n).unwrap();
            Roll(proc() {
                factorial(n - 1, nb * acc)
            })
        }
    }

    #[test]
    fn welp() {
        let acc: BigInt = FromPrimitive::from_uint(1u).unwrap();
        println!("{}", factorial(1500, acc).go(force))
    }

}
