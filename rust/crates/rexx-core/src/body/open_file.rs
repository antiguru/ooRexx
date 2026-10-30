/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

/// How many written bytes a buffered stream holds before it writes them: the
/// oracle's `DEFAULT_BUFFER_SIZE` (`common/platform/unix/SysFile.hpp`).
pub const STREAM_BUFFER: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteBuffering {
    None,
    Positioned,
    StandardFirstWrite,
    Standard,
}

/// The descriptor an open stream holds, with the logical positions one
/// descriptor serves. The character positions are 1-based "next byte", as the
/// oracle's are. A SEEK that would move one below 1 fails, so a write
/// position is 0 only on a stream opened READ, which has none.
///
/// A stream is buffered unless it was opened `NOBUFFER` or is a terminal
/// (rexxref streamclasses.xml, `NOBUFFER`): its writes reach the file when
/// [`STREAM_BUFFER`] bytes have gathered, when the next write does not follow
/// on, and before anything reads, measures or positions the descriptor
/// through [`OpenFile::flushed`], so another stream or process sees them only
/// from then on. A drop writes what is left.
#[derive(Debug)]
pub struct OpenFile {
    file: std::fs::File,
    /// Written bytes the file does not hold yet, which land at byte offset
    /// `held_at` (where the file stands, on a transient stream or standard
    /// output). Standard output retains `held_at` as its buffered position.
    held: Vec<u8>,
    held_at: u64,
    buffering: WriteBuffering,
    pub read_position: i64,
    pub write_position: i64,
    /// A byte taken from a transient stream to learn whether data waits, which
    /// the next read returns first. Only a device that cannot say how much
    /// waits without being read leaves one.
    pub pending: Option<u8>,
    pub transient: bool,
    /// A character position and the line it lies in, from the last line
    /// operation. Any write landing before the position drops it, so it is
    /// always what a count from the start would answer.
    pub line_mark: Option<(i64, i64)>,
    /// The stream's line count, dropped by any write.
    pub line_total: Option<u64>,
    /// Debug builds only: digests of the bytes [`OpenFile::line_mark`] and
    /// [`OpenFile::line_total`] were counted over, so a check of either runs
    /// only while those bytes are unchanged. Another stream object or process
    /// writing the file may leave the line answers stale (rexxref
    /// iostrms.xml:495-500); the checks then stand aside.
    pub line_mark_digest: Option<u64>,
    pub line_total_digest: Option<u64>,
}

impl OpenFile {
    /// `file` opened, read from position 1 and written at `write_position`.
    pub fn new(
        file: std::fs::File,
        write_position: i64,
        transient: bool,
        buffered: bool,
    ) -> OpenFile {
        OpenFile {
            file,
            held: Vec::new(),
            held_at: 0,
            buffering: if buffered {
                WriteBuffering::Positioned
            } else {
                WriteBuffering::None
            },
            read_position: 1,
            write_position,
            pending: None,
            transient,
            line_mark: None,
            line_total: None,
            line_mark_digest: None,
            line_total_digest: None,
        }
    }

    /// An owned duplicate of a persistent standard output descriptor, with
    /// independent logical positions and a buffer written at the shared offset.
    pub fn standard_output(file: std::fs::File, buffered: bool) -> OpenFile {
        let mut open = Self::new(file, 1, false, false);
        open.reopen_standard_output(buffered);
        open
    }

    /// Reopens already flushed standard output with its retained positions.
    pub fn reopen_standard_output(&mut self, buffered: bool) {
        debug_assert!(self.held.is_empty());
        self.buffering = if buffered {
            WriteBuffering::StandardFirstWrite
        } else {
            WriteBuffering::None
        };
    }

    /// Writes standard output at its shared descriptor offset and updates this
    /// stream's logical write position, without seeking on subsequent flushes.
    pub fn write_standard_output(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        if self.buffering == WriteBuffering::None {
            self.file.write_all(bytes)?;
        } else {
            if self.buffering == WriteBuffering::StandardFirstWrite && !bytes.is_empty() {
                // SysFile switches its first buffered write from its cached
                // read position. Later flushes follow the shared descriptor.
                self.file.seek(SeekFrom::Start(self.held_at))?;
                self.buffering = WriteBuffering::Standard;
            }
            self.write(self.held_at + self.held.len() as u64, bytes)?;
        }
        self.write_position = self.offset() as i64 + 1;
        Ok(())
    }

    /// The descriptor as it stands, without the writes still held: for its
    /// type, its number and its size where held writes cannot matter.
    pub fn file(&self) -> &std::fs::File {
        &self.file
    }

    /// The descriptor once the held writes are in it, for anything that
    /// reads, measures or positions it. A failed write is dropped here, as
    /// the oracle's `SysFile::flush` from a read drops it.
    pub fn flushed(&mut self) -> &mut std::fs::File {
        let _ = self.flush();
        &mut self.file
    }

    /// The size the file has once the held writes are in it, without
    /// writing them.
    pub fn size(&self) -> u64 {
        let size = self.file.metadata().map_or(0, |meta| meta.len());
        match self.held.is_empty() || self.transient {
            true => size,
            false => size.max(self.held_at + self.held.len() as u64),
        }
    }

    /// The descriptor's byte offset as the stream sees it: past the held
    /// writes, which have not moved the descriptor yet.
    pub fn offset(&mut self) -> u64 {
        use std::io::Seek;
        if self.buffering == WriteBuffering::StandardFirstWrite {
            return self.held_at;
        }
        match self.held.is_empty() || self.transient {
            true => self.file.stream_position().unwrap_or(0),
            false => self.held_at + self.held.len() as u64,
        }
    }

    /// Writes the held bytes.
    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.held.is_empty() {
            return Ok(());
        }
        let held = std::mem::take(&mut self.held);
        self.write_through(self.held_at, &held)
    }

    /// Writes `bytes` at byte offset `offset` (where the file stands, on a
    /// transient stream), as `SysFile::write` does: held while they follow
    /// on from the held bytes and fit, written in buffer-sized pieces as the
    /// buffer fills, and written at once, after the held bytes, when they
    /// are more than a buffer holds.
    pub fn write(&mut self, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
        if self.buffering == WriteBuffering::None {
            return self.write_through(offset, bytes);
        }
        let follows = self.transient || self.held_at + self.held.len() as u64 == offset;
        if !follows || bytes.len() > STREAM_BUFFER {
            self.flush()?;
        }
        if bytes.len() > STREAM_BUFFER {
            return self.write_through(offset, bytes);
        }
        if self.held.is_empty() {
            self.held_at = offset;
        }
        let mut rest = bytes;
        while !rest.is_empty() {
            if self.held.len() == STREAM_BUFFER {
                let at = self.held_at + STREAM_BUFFER as u64;
                self.flush()?;
                self.held_at = at;
            }
            let take = rest.len().min(STREAM_BUFFER - self.held.len());
            self.held.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
        }
        Ok(())
    }

    fn write_through(&mut self, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
        use std::io::{Seek, SeekFrom, Write};
        let standard = matches!(
            self.buffering,
            WriteBuffering::Standard | WriteBuffering::StandardFirstWrite
        );
        if !self.transient && !standard {
            self.file.seek(SeekFrom::Start(offset))?;
        }
        self.file.write_all(bytes)?;
        if standard {
            self.held_at += bytes.len() as u64;
        }
        Ok(())
    }
}

/// Writes what is held, except while a panic unwinds: a crash loses held
/// writes, as the oracle's does.
impl Drop for OpenFile {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let _ = self.flush();
        }
    }
}
