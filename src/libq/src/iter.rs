pub trait SplitOn
where
    Self: Iterator,
    Self: Sized,
    Self::Item: PartialEq,
{
    fn split_on(self, val: Self::Item, inclusive: bool) -> SplitOnIter<Self>;

    fn split_on_inclusive(self, val: Self::Item) -> SplitOnIter<Self> {
        return self.split_on(val, true);
    }

    fn split_on_exclusive(self, val: Self::Item) -> SplitOnIter<Self> {
        return self.split_on(val, false);
    }
}

impl<I: Iterator + Sized> SplitOn for I
where
    I::Item: PartialEq,
{
    fn split_on(self, val: Self::Item, inclusive: bool) -> SplitOnIter<Self> {
        return SplitOnIter::new(self, val, inclusive);
    }
}

pub struct SplitOnIter<I>
where
    I: Iterator,
    I::Item: PartialEq,
{
    iter: I,
    val: I::Item,
    inclusive: bool,
}

impl<I: Iterator> SplitOnIter<I>
where
    I::Item: PartialEq,
{
    pub fn new(iter: I, val: I::Item, inclusive: bool) -> SplitOnIter<I> {
        return SplitOnIter {
            iter: iter,
            val: val,
            inclusive: inclusive,
        };
    }
}

impl<I: Iterator> Iterator for SplitOnIter<I>
where
    I::Item: PartialEq,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Vec<I::Item>> {
        let mut values = Vec::new();
        loop {
            if let Some(val) = self.iter.next() {
                if val == self.val {
                    if self.inclusive {
                        values.push(val);
                    }

                    return Some(values);
                }

                values.push(val);
            } else {
                return None;
            }
        }
    }
}
