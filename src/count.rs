use std::ops::Range;

use crate::libs::opencl::CACHE_COUNT;

pub trait AsRangeArg {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

impl AsRangeArg for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl AsRangeArg for usize {
    fn start(&self) -> usize {
        0
    }

    fn end(&self) -> usize {
        *self
    }
}

impl AsRangeArg for (usize, usize) {
    fn start(&self) -> usize {
        self.0
    }

    fn end(&self) -> usize {
        self.1
    }
}

//inclusive range
pub fn range<R: AsRangeArg>(range: R) -> Count {
    Count(range.start(), range.end())
}

pub struct Count(usize, usize);

pub struct CountIntoIter {
    epoch: usize,
    idx: usize,
    end: usize,
}

impl Iterator for CountIntoIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { CACHE_COUNT = self.idx };
        if self.epoch > self.end {
            return None;
        }
        let epoch = Some(self.epoch);
        self.epoch += 1;
        epoch
    }
}

impl IntoIterator for Count {
    type Item = usize;

    type IntoIter = CountIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        CountIntoIter {
            epoch: self.0,
            idx: unsafe { CACHE_COUNT },
            end: self.1
        }
    }
}
