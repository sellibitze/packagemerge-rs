use std::mem;

#[derive(Copy,Clone,PartialEq,Eq,Debug)]
pub enum Either<T, U> {
    Left(T),
    Right(U),
}

pub enum Pick {
    Left,
    Right,
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MergeIter<A: Iterator, B: Iterator, C> {
    ita: A,
    itb: B,
    ela: Option<A::Item>,
    elb: Option<B::Item>,
    pck: C,
}

impl<A: Iterator, B: Iterator, C> MergeIter<A, B, C>
where C: FnMut(&A::Item, &B::Item) -> Pick {
    fn new(mut ia: A, mut ib: B, pck: C) -> Self {
        let ea = ia.next();
        let eb = ib.next();
        MergeIter {
            ita: ia,
            itb: ib,
            ela: ea,
            elb: eb,
            pck: pck
        }
    }
}

impl<A: Iterator, B: Iterator, C> Iterator for MergeIter<A, B, C>
where C: FnMut(&A::Item, &B::Item) -> Pick {
    type Item = Either<A::Item, B::Item>;

    fn next(&mut self) -> Option<Either<A::Item, B::Item>> {
        match (&mut self.ela, &mut self.elb) {
            (&mut None, &mut None) => None,
            (a @ &mut Some(_), &mut None) => {
                let tmp = self.ita.next();
                Some(Either::Left(mem::replace(a, tmp).unwrap()))
            }
            (&mut None, b @ &mut Some(_)) => {
                let tmp = self.itb.next();
                Some(Either::Right(mem::replace(b, tmp).unwrap()))
            }
            (a @ &mut Some(_), b @ &mut Some(_)) => {
                let p = (self.pck)(a.as_ref().unwrap(), b.as_ref().unwrap());
                match p {
                    Pick::Left => {
                        let tmp = self.ita.next();
                        Some(Either::Left(mem::replace(a, tmp).unwrap()))
                    },
                    Pick::Right => {
                        let tmp = self.itb.next();
                        Some(Either::Right(mem::replace(b, tmp).unwrap()))
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut add = 0;
        if self.ela.is_some() { add += 1; }
        if self.elb.is_some() { add += 1; }
        let (min1, max1) = self.ita.size_hint();
        let (min2, max2) = self.itb.size_hint();
        let upper = match (max1, max2) {
            (Some(a), Some(b)) => Some(a + b + add),
            _ => None
        };
        (min1 + min2 + add, upper)
    }
}

impl<A: ExactSizeIterator, B: ExactSizeIterator, C> ExactSizeIterator
for MergeIter<A, B, C> where C: FnMut(&A::Item, &B::Item) -> Pick {
}


pub fn merge<I1: Iterator, I2: Iterator, P>(a: I1, b: I2, pick: P) -> MergeIter<I1, I2, P>
where P: FnMut(&I1::Item, &I2::Item) -> Pick {
    MergeIter::new(a, b, pick)
}

#[cfg(test)]
mod tests {
    use super::{ Pick, Either, merge };

    fn pick_f64(&a: &f64, &b: &f64) -> Pick {
        if a < b { Pick::Left }
        else { Pick::Right }
    }
    
    #[test]
    fn it_works() {
        let f1 = [1.25, 2.375, 5.5, 9.25];
        let f2 = [3.25, 3.375, 6.5, 7.75];
        let vec: Vec<_> = merge(f1.iter().cloned(), f2.iter().cloned(), pick_f64).collect();
        assert_eq!(&vec[..], &[
            Either::Left(1.25),
            Either::Left(2.375),
            Either::Right(3.25),
            Either::Right(3.375),
            Either::Left(5.5),
            Either::Right(6.5),
            Either::Right(7.75),
            Either::Left(9.25)
        ]);
    }
}

