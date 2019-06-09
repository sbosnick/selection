// Copyright 2019 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{Layout, Result};
use std::ops::Range;

/// A potentially parallelizable writer for the output file. Created by the
/// [`output`][Layout::output] method.
pub struct OutputWriter<'a, 'b>(ElfWriter<'a, 'b, Layout<'a>>);

impl<'a, 'b> OutputWriter<'a, 'b> {
    pub(crate) fn new(layout: &'a Layout<'a>, output: &'b mut [u8]) -> OutputWriter<'a, 'b> {
        OutputWriter(ElfWriter {
            output,
            segments: 0..layout.out_segments(),
            writer: layout,
        })
    }

    /// Potentially split this `OutputWriter` into two independent `OutputWriter`'s
    /// for separate parts of the output.
    ///
    /// The signature of this function is intended to make it usable as the
    /// `splitter` argument to the [rayon][rayon] [split] function.
    ///
    /// [rayon]: https://crates.io/crates/rayon
    /// [split]: https://docs.rs/rayon/1.0.3/rayon/iter/fn.split.html
    pub fn split(self) -> (Self, Option<Self>) {
        let (l, r) = self.0.split();
        (OutputWriter(l), r.map(|ew| OutputWriter(ew)))
    }

    /// Write the portion of the output represented by this `OutputWriter` to the
    /// corresponding sub-slice of the output bytes passed to the
    /// [`output`][Layout::output] method.
    pub fn write(&mut self) -> Result<()> {
        self.0.write()
    }
}

trait SegmentWriter {
    fn segment_size(&self, segment: usize) -> usize;
    fn write_segment<'b>(&self, segment: usize, output: &'b mut [u8]) -> Result<()>;
}

impl<'a> SegmentWriter for Layout<'a> {
    fn segment_size(&self, segment: usize) -> usize {
        self.segment_size(segment)
    }

    fn write_segment<'b>(&self, segment: usize, output: &'b mut [u8]) -> Result<()> {
        self.write_segment(segment, output)
    }
}

struct ElfWriter<'a, 'b, S> {
    segments: Range<usize>,
    writer: &'a S,
    output: &'b mut [u8],
}

impl<'a, 'b, S: SegmentWriter> ElfWriter<'a, 'b, S> {
    fn split(self) -> (Self, Option<Self>) {
        if self.segments.len() <= 1 {
            (self, None)
        } else {
            let mid = self.segments.start + self.segments.len() / 2;
            let mut mid_output = 0;
            for segment in self.segments.start..mid {
                mid_output += self.writer.segment_size(segment);
            }
            let (lout, rout) = self.output.split_at_mut(mid_output);

            (
                ElfWriter {
                    segments: self.segments.start..mid,
                    writer: self.writer,
                    output: lout,
                },
                Some(ElfWriter {
                    segments: mid..self.segments.end,
                    writer: self.writer,
                    output: rout,
                }),
            )
        }
    }

    fn write(&mut self) -> Result<()> {
        let mut offset = 0;

        for segment in self.segments.clone() {
            let size = self.writer.segment_size(segment);
            self.writer
                .write_segment(segment, &mut self.output[offset..offset + size])?;

            offset += size;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cell::RefCell;

    struct FakeWriter {
        segment_sizes: Vec<usize>,
        segments_sized: RefCell<Vec<usize>>,
        segments_writen: RefCell<Vec<usize>>,
        output_sizes: RefCell<Vec<usize>>,
    }

    impl FakeWriter {
        fn new(segment_sizes: Vec<usize>) -> FakeWriter {
            FakeWriter {
                segment_sizes,
                segments_sized: RefCell::new(Vec::new()),
                segments_writen: RefCell::new(Vec::new()),
                output_sizes: RefCell::new(Vec::new()),
            }
        }
    }

    impl SegmentWriter for FakeWriter {
        fn segment_size(&self, segment: usize) -> usize {
            self.segments_sized.borrow_mut().push(segment);
            self.segment_sizes[segment]
        }

        fn write_segment<'b>(&self, segment: usize, output: &'b mut [u8]) -> Result<()> {
            self.segments_writen.borrow_mut().push(segment);
            self.output_sizes.borrow_mut().push(output.len());
            Ok(())
        }
    }

    #[test]
    fn elf_writer_write_queries_segment_size() {
        let fw = FakeWriter::new(vec![3000, 4000]);
        let mut output = vec![0; 10000];

        let mut sut = ElfWriter {
            segments: 0..2,
            writer: &fw,
            output: &mut output,
        };
        sut.write().unwrap();

        assert!(fw.segments_sized.borrow().contains(&0));
        assert!(fw.segments_sized.borrow().contains(&1));
    }

    #[test]
    fn elf_writer_write_writes_expected_segments() {
        let fw = FakeWriter::new(vec![3000, 4000]);
        let mut output = vec![0; 10000];

        let mut sut = ElfWriter {
            segments: 0..2,
            writer: &fw,
            output: &mut output,
        };
        sut.write().unwrap();

        assert!(fw.segments_writen.borrow().contains(&0));
        assert!(fw.segments_writen.borrow().contains(&1));
    }

    #[test]
    fn elf_writer_write_writes_with_expected_size_output() {
        let size1 = 3000;
        let size2 = 4000;
        let fw = FakeWriter::new(vec![size1, size2]);
        let mut output = vec![0; 10000];

        let mut sut = ElfWriter {
            segments: 0..2,
            writer: &fw,
            output: &mut output,
        };
        sut.write().unwrap();

        assert!(fw.output_sizes.borrow().contains(&size1));
        assert!(fw.output_sizes.borrow().contains(&size2));
    }

    #[test]
    fn elf_writer_split_single_segment_is_noop() {
        let fw = FakeWriter::new(vec![3000]);
        let mut output = vec![0; 10000];

        let sut = ElfWriter {
            segments: 0..1,
            writer: &fw,
            output: &mut output,
        };
        let (l, r) = sut.split();

        assert!(r.is_none());
        assert_eq!(l.segments, 0..1);
        assert_eq!(l.output as *const _, (&mut output as &mut [u8]) as *const _);
    }

    #[test]
    fn elf_writer_split_queries_segment_sizes() {
        let fw = FakeWriter::new(vec![3000, 4000, 1000, 500]);
        let mut output = vec![0; 10000];

        let sut = ElfWriter {
            segments: 0..4,
            writer: &fw,
            output: &mut output,
        };
        let _ = sut.split();

        assert!(fw.segments_sized.borrow().contains(&0));
        assert!(fw.segments_sized.borrow().contains(&1));
    }

    #[test]
    fn elf_writer_split_splits_segments() {
        let fw = FakeWriter::new(vec![3000, 4000, 1000, 500]);
        let mut output = vec![0; 10000];

        let sut = ElfWriter {
            segments: 0..4,
            writer: &fw,
            output: &mut output,
        };
        let (l, r) = sut.split();
        let r = r.expect("r is unexpectly None");

        assert_eq!(l.segments, 0..2);
        assert_eq!(r.segments, 2..4);
    }

    #[test]
    fn elf_writer_split_splits_output() {
        let fw = FakeWriter::new(vec![3000, 4000, 1000, 500]);
        let mut output = vec![0; 10000];
        let output_ptr = output.as_ptr();

        let sut = ElfWriter {
            segments: 0..4,
            writer: &fw,
            output: &mut output,
        };
        let (l, r) = sut.split();
        let r = r.expect("r is unexpectly None");

        assert_eq!(l.output.as_ptr(), output_ptr);
        let one_past_end = unsafe { l.output.as_ptr().offset(l.output.len() as isize) };
        assert_eq!(one_past_end, r.output.as_ptr());
        assert_eq!(l.output.len() + r.output.len(), output.len());
    }
}
