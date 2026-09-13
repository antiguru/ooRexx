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

//! `ADDRESS ... WITH`: the permanent per-environment configuration a program
//! sets, and the per-command context saying where one command's three streams
//! go.
//!
//! The oracle splits these the same way. A `CommandIOConfiguration` is what
//! the parser built and what an `ADDRESS env WITH ...` stores under the
//! environment name; a `CommandIOContext` is built at **issue** time by
//! evaluating that configuration's sources and targets in the variable
//! context then in force.

use std::rc::Rc;

use rexx_core::{Body, Decoded, ObjRef};
use rexx_parse::{AddressIo, Expr, OutputOption, Redirection, SymbolId};

use crate::activation::IoConfigs;
use crate::error::{Failure, Raised};
use crate::{Code, Interp, Loud};

/// Which of a command's three streams a redirection names -- the word its
/// `>K>` line carries and the one its errors are worded for.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Stream {
    Input,
    Output,
    Error,
}

impl Stream {
    fn keyword(self) -> &'static str {
        match self {
            Stream::Input => "INPUT",
            Stream::Output => "OUTPUT",
            Stream::Error => "ERROR",
        }
    }
}

/// One command's resolved redirection.
pub(crate) struct IoContext {
    /// Everything the child reads on standard input, gathered **before it
    /// starts**. `readInputBuffered` does the same, and that is what lets one
    /// object be a command's input and its output at once: the doc's `sort`
    /// example reads the array before the target empties it.
    pub(crate) input: Option<Vec<u8>>,
    output: Option<Target>,
    error: Option<Target>,
}

/// Where one redirected output stream's lines end up. Every write happens at
/// [`IoContext::finish`] rather than as the bytes arrive, which is what keeps
/// a target that is also the input source readable until the child has run.
enum Target {
    /// A stem object, whose tails this rewrites.
    Stem { object: ObjRef, append: bool },
    /// An `OrderedCollection`: `EMPTY`, then one `APPEND` per line, so the
    /// class's own methods decide what that means.
    Collection { object: ObjRef, append: bool },
    /// An open stream, one `LINEOUT` per line.
    ///
    /// `qualified` is `Some` for a `STREAM name` target, which this opened and
    /// therefore closes, and `None` for a `USING` stream object, which the
    /// program owns and which stays open -- measured, such an object's
    /// `~state` is still `READY` after the command while its bytes are on
    /// disk.
    Stream {
        object: ObjRef,
        qualified: Option<Vec<u8>>,
    },
}

impl Target {
    fn object(&self) -> ObjRef {
        match self {
            Target::Stem { object, .. }
            | Target::Collection { object, .. }
            | Target::Stream { object, .. } => *object,
        }
    }

    /// Whether two targets are one destination.
    ///
    /// **Two named streams compare on the qualified name, not on identity**,
    /// which is `StreamOutputTarget::isSameTarget`'s own override -- measured,
    /// `g.txt` with `g.txt` and `h.txt` with `./h.txt` each collapse to a
    /// single file the two streams interleave into.
    fn same_target(&self, other: &Target) -> bool {
        match (self, other) {
            (
                Target::Stream {
                    qualified: Some(one),
                    ..
                },
                Target::Stream {
                    qualified: Some(two),
                    ..
                },
            ) => one == two,
            (
                Target::Stream {
                    qualified: Some(_), ..
                },
                _,
            )
            | (
                _,
                Target::Stream {
                    qualified: Some(_), ..
                },
            ) => false,
            _ => self.object() == other.object(),
        }
    }
}

impl IoContext {
    pub(crate) fn redirects_output(&self) -> bool {
        self.output.is_some()
    }

    pub(crate) fn redirects_error(&self) -> bool {
        self.error.is_some()
    }

    /// Whether both output streams resolved to one object, in which case the
    /// child writes them down a single pipe and they interleave in the order
    /// it wrote them -- measured, `echo out1; echo err1 1>&2; echo out2` with
    /// both streams on one stem fills it with `out1`, `err1`, `out2`.
    pub(crate) fn shares_one_target(&self) -> bool {
        match (&self.output, &self.error) {
            (Some(output), Some(error)) => output.same_target(error),
            _ => false,
        }
    }

    /// Splits what the child wrote and hands it to the targets.
    ///
    /// `out` is standard output and `err` standard error; under
    /// [`IoContext::shares_one_target`] the caller has already merged them
    /// into `out` and `err` is empty.
    pub(crate) fn finish(
        &self,
        interp: &mut Interp,
        out: &[u8],
        err: &[u8],
    ) -> Result<(), Failure> {
        if let Some(target) = &self.output {
            let lines = split_lines(out);
            interp.write_target(target, &lines)?;
        }
        // **One object for both streams is written once.** Everything the
        // child produced came down the output pipe and `err` is empty, so a
        // second write here would replace what the first one just landed.
        if self.shares_one_target() {
            return Ok(());
        }
        if let Some(target) = &self.error {
            let lines = split_lines(err);
            interp.write_target(target, &lines)?;
        }
        Ok(())
    }
}

/// One captured stream as lines.
///
/// `\n` ends a line and a `\r` immediately before it is part of that one
/// terminator; a `\r` anywhere else is data, as a NUL always is; and a
/// trailing remainder with no terminator is a line of its own. Measured:
/// `printf "a\nb"` gives two, `printf "a\r\nb\r\n"` gives two whose first is
/// one byte long, `printf "a\0b\n"` gives one of three bytes, and
/// `printf ""` gives none.
fn split_lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at..].iter().position(|byte| *byte == b'\n') {
            Some(offset) => {
                let mut end = at + offset;
                if end > at && bytes[end - 1] == b'\r' {
                    end -= 1;
                }
                lines.push(bytes[at..end].to_vec());
                at += offset + 1;
            }
            None => {
                lines.push(bytes[at..].to_vec());
                break;
            }
        }
    }
    lines
}

/// One stream's effective redirection and the option that came with it.
///
/// **The local configuration decides only where it says something.**
/// `NORMAL` is a reset rather than silence, a local target replaces the
/// permanent one, and a stream the local configuration does not mention keeps
/// the permanent target -- measured, a permanent `with output stem o.` and a
/// one-off `with error stem e.` fill both. The option travels with whichever
/// configuration supplied the target, as it does in `createOutputTarget`.
fn merge<'a>(
    local: Option<(&'a Redirection, OutputOption)>,
    global: Option<(&'a Redirection, OutputOption)>,
) -> Option<(&'a Redirection, OutputOption)> {
    match local {
        Some((Redirection::Normal, _)) => None,
        Some((Redirection::Default, _)) | None => match global {
            Some((Redirection::Default | Redirection::Normal, _)) | None => None,
            Some(found) => Some(found),
        },
        Some(found) => Some(found),
    }
}

impl Interp {
    /// The permanent configuration stored for `environment`, looked up by the
    /// **upcased** name -- measured, `address sh with output stem o.` still
    /// redirects after a later `address SH`.
    fn io_config(&self, environment: &[u8]) -> Option<Rc<AddressIo>> {
        let upper = environment.to_ascii_uppercase();
        self.activation()
            .io_configs
            .as_ref()?
            .get(upper.as_slice())
            .cloned()
    }

    /// `addIOConfig`: makes `io` the configuration for `environment`.
    ///
    /// The table is cloned first when this activation inherited one, which is
    /// `checkIOConfigTable`'s lazy copy and what keeps a callee's write off
    /// its caller.
    pub(crate) fn add_io_config(&mut self, environment: &[u8], io: Rc<AddressIo>) {
        let upper = environment.to_ascii_uppercase().into_boxed_slice();
        let table = self
            .activation_mut()
            .io_configs
            .get_or_insert_with(|| Rc::new(IoConfigs::default()));
        Rc::make_mut(table).insert(upper, io);
    }

    /// `setAddress`' second half: the `WITH` configuration an `ADDRESS`
    /// instruction carried becomes the one stored under the name it just set.
    ///
    /// An instruction with no `WITH` stores nothing and leaves whatever that
    /// name already holds in place.
    pub(crate) fn store_io_config(&mut self, environment: &[u8], address: &rexx_parse::Address) {
        if let Some(io) = &address.io {
            self.add_io_config(environment, Rc::new(io.as_ref().clone()));
        }
    }

    /// `resolveAddressIOConfig`: the context one command runs under, or
    /// `None` where neither the environment nor the instruction redirects
    /// anything.
    ///
    /// Sources and targets are evaluated here, at issue time and in the
    /// variable context in force -- so a permanent configuration inherited by
    /// a callee fills the **callee's** own stem, measured. The order is
    /// `INPUT`, `OUTPUT`, `ERROR`, which their `>K>` lines show.
    pub(crate) fn io_context(
        &mut self,
        code: &Code<'_>,
        environment: &[u8],
        local: Option<&AddressIo>,
    ) -> Result<Option<IoContext>, Failure> {
        let global = self.io_config(environment);
        let global = global.as_deref();
        if global.is_none() && local.is_none() {
            return Ok(None);
        }
        let input = merge(
            local.map(|io| (&io.input, OutputOption::Default)),
            global.map(|io| (&io.input, OutputOption::Default)),
        );
        let output = merge(
            local.map(|io| (&io.output, io.output_option)),
            global.map(|io| (&io.output, io.output_option)),
        );
        let error = merge(
            local.map(|io| (&io.error, io.error_option)),
            global.map(|io| (&io.error, io.error_option)),
        );
        let input = match input {
            Some((source, _)) => Some(self.input_buffer(code, source)?),
            None => None,
        };
        let output = match output {
            Some((target, option)) => {
                Some(self.output_target(code, target, option, Stream::Output)?)
            }
            None => None,
        };
        let error = match error {
            Some((target, option)) => {
                Some(self.output_target(code, target, option, Stream::Error)?)
            }
            None => None,
        };
        Ok(Some(IoContext {
            input,
            output,
            error,
        }))
    }

    /// Everything a redirected standard input is fed, as one buffer.
    ///
    /// Every line carries a terminator, the last one included -- measured,
    /// `'cat -A' with input using 'x'` prints `x$`.
    fn input_buffer(&mut self, code: &Code<'_>, source: &Redirection) -> Result<Vec<u8>, Failure> {
        let lines = match source {
            // `merge` answers `None` for both, so neither reaches here.
            Redirection::Default | Redirection::Normal => Vec::new(),
            Redirection::Stem(id) => {
                let object = self.redirect_stem(code, *id, Stream::Input);
                self.stem_input_lines(object)?
            }
            Redirection::Using(expression) => {
                let object = self.eval(code, expression)?;
                self.roots.push_temp(object);
                self.trace_redirect_target(Stream::Input, object);
                self.using_input_lines(object)?
            }
            Redirection::Stream(expression) => {
                let name = self.redirect_stream_name(code, expression, Stream::Input)?;
                let stream = self.open_named_stream(&name, b"READ", true)?;
                let lines = self.read_stream_lines(stream)?;
                let caller = self.caller();
                self.send_message(stream, b"CLOSE", None, &[], caller)?;
                lines
            }
        };
        let mut buffer = Vec::new();
        for line in lines {
            buffer.extend_from_slice(&line);
            buffer.push(b'\n');
        }
        Ok(buffer)
    }

    /// A stem read as input: `stem.1` through `stem.<stem.0>`.
    ///
    /// A tail with no value of its own contributes the stem's **default**,
    /// and with no default its own derived name -- measured, a stem holding
    /// `IN.1` and `IN.3` with `IN.0 = 3` feeds `one`, `IN.` and `three`,
    /// which is `getFullElement`'s ordinary read.
    fn stem_input_lines(&mut self, object: ObjRef) -> Result<Vec<Vec<u8>>, Failure> {
        let Some(count) = self.redirect_stem_count(object)? else {
            let name = self.stem_object_name(object);
            return Err(Raised::stem_without_a_size(&name).into());
        };
        let mut lines = Vec::with_capacity(count);
        for index in 1..=count {
            let key = index.to_string().into_bytes();
            let value = self.stem_object_read(object, &key);
            lines.push(self.string_value_text(value));
        }
        Ok(lines)
    }

    /// `WITH INPUT USING expr`, whose object decides what it means.
    ///
    /// The order is `createInputSource`'s own: a String is one line; a stem
    /// is read as `STEM` reads one; an Array contributes its items and skips
    /// its holes; anything else is asked for an array and is 98.924 if it
    /// does not answer one -- measured, `.nil` and `42` are the two ends of
    /// that, `42` being a String.
    fn using_input_lines(&mut self, object: ObjRef) -> Result<Vec<Vec<u8>>, Failure> {
        if is_string(self, object) {
            return Ok(vec![self.string_value_text(object)]);
        }
        if self.is_stem_object(object) {
            return self.stem_input_lines(object);
        }
        if self.is_rexx_queue(object) {
            return Err(Loud::redirection("RexxQueue", "Phase 10").into());
        }
        if self.is_stream_object(object) {
            return self.read_stream_lines(object);
        }
        if let Some(path) = self.file_object_path(object)? {
            let stream = self.open_named_stream(&path, b"READ", true)?;
            let lines = self.read_stream_lines(stream)?;
            let caller = self.caller();
            self.send_message(stream, b"CLOSE", None, &[], caller)?;
            return Ok(lines);
        }
        let array = match self.array_slots_of(object) {
            Some(_) => object,
            None => {
                let caller = self.caller();
                let selector = self.text(b"ARRAY");
                self.roots.push_temp(selector);
                let answered = self
                    .send_message(object, b"REQUEST", None, &[Some(selector)], caller)?
                    .unwrap_or(ObjRef::NIL);
                self.roots.push_temp(answered);
                answered
            }
        };
        let Some(slots) = self.array_slots_of(array) else {
            let shown = self.string_value_text(object);
            return Err(Raised::address_input_source(&shown).into());
        };
        let mut lines = Vec::new();
        for item in slots.into_iter().flatten() {
            lines.push(self.string_value_text(item));
        }
        Ok(lines)
    }

    /// `WITH OUTPUT`/`ERROR`, whose target is resolved but not yet written.
    fn output_target(
        &mut self,
        code: &Code<'_>,
        target: &Redirection,
        option: OutputOption,
        stream: Stream,
    ) -> Result<Target, Failure> {
        let append = option == OutputOption::Append;
        match target {
            // `merge` answers `None` for both, so neither reaches here.
            Redirection::Default | Redirection::Normal => {
                unreachable!("a stream `merge` answered `None` for reached the target builder")
            }
            Redirection::Stem(id) => {
                let object = self.redirect_stem(code, *id, stream);
                Ok(Target::Stem { object, append })
            }
            Redirection::Using(expression) => {
                let object = self.eval(code, expression)?;
                self.roots.push_temp(object);
                self.trace_redirect_target(stream, object);
                self.using_output_target(object, option)
            }
            Redirection::Stream(expression) => {
                let name = self.redirect_stream_name(code, expression, stream)?;
                let mode: &[u8] = match append {
                    true => b"WRITE APPEND",
                    false => b"WRITE REPLACE",
                };
                let object = self.open_named_stream(&name, mode, false)?;
                Ok(Target::Stream {
                    object,
                    qualified: Some(name),
                })
            }
        }
    }

    /// `WITH OUTPUT USING expr`, in `createOutputTarget`'s own order: a stem,
    /// then a stream-shaped object, then an `OrderedCollection`, and 98.996
    /// for anything else.
    fn using_output_target(
        &mut self,
        object: ObjRef,
        option: OutputOption,
    ) -> Result<Target, Failure> {
        let append = option == OutputOption::Append;
        // **Whether an option was written matters here, not just which one.**
        // Neither `REPLACE` nor `APPEND` means anything to an object that is
        // not a file, and the oracle refuses both rather than ignoring them.
        let optioned = option != OutputOption::Default;
        if self.is_stem_object(object) {
            return Ok(Target::Stem { object, append });
        }
        if self.is_rexx_queue(object) {
            if optioned {
                return Err(Raised::queue_target_option().into());
            }
            return Err(Loud::redirection("RexxQueue", "Phase 10").into());
        }
        if self.is_stream_object(object) {
            if optioned {
                return Err(Raised::stream_target_option().into());
            }
            return Ok(Target::Stream {
                object,
                qualified: None,
            });
        }
        if let Some(path) = self.file_object_path(object)? {
            let mode: &[u8] = match append {
                true => b"WRITE APPEND",
                false => b"WRITE REPLACE",
            };
            let opened = self.open_named_stream(&path, mode, false)?;
            return Ok(Target::Stream {
                object: opened,
                qualified: Some(path),
            });
        }
        if self.is_ordered_collection(object) {
            return Ok(Target::Collection { object, append });
        }
        let shown = self.string_value_text(object);
        Err(Raised::address_output_target(&shown).into())
    }

    /// The stem object a `STEM name.` redirection names, read where the
    /// command is issued.
    ///
    /// The read echoes as an ordinary variable read before the `>K>` line --
    /// measured under `trace i`, `>V> OO. => "OO."` then
    /// `>K> "OUTPUT" => "OO."`.
    fn redirect_stem(&mut self, code: &Code<'_>, id: SymbolId, stream: Stream) -> ObjRef {
        let name = code.symbols.name(id).as_bytes().to_vec();
        let object = self.read_stem(&name);
        self.roots.push_temp(object);
        self.echo_symbol_read(code, id, object);
        self.trace_redirect_target(stream, object);
        object
    }

    /// `traceKeywordResult`: the `>K>` line naming an evaluated target.
    /// Gated on the same flag [`Interp::trace_keyword`] is, because the
    /// string value it shows can cost a `makeString` send.
    fn trace_redirect_target(&mut self, stream: Stream, value: ObjRef) {
        if !self.trace_mode().results {
            return;
        }
        let text = self.string_value_text(value);
        let indent = self.clause_state.current_value_indent;
        self.trace_keyword(indent, stream.keyword(), &text);
    }

    /// Writes one target's lines.
    fn write_target(&mut self, target: &Target, lines: &[Vec<u8>]) -> Result<(), Failure> {
        match *target {
            Target::Stem { object, append } => self.write_stem_target(object, append, lines),
            Target::Collection { object, append } => {
                self.write_collection_target(object, append, lines)
            }
            Target::Stream { object, .. } => {
                let close = matches!(
                    target,
                    Target::Stream {
                        qualified: Some(_),
                        ..
                    }
                );
                self.write_stream_target(object, close, lines)
            }
        }
    }

    /// `StreamOutputTarget`: one `LINEOUT` per line, then a `CLOSE` for a
    /// target this redirection opened. A `USING` stream object is left open,
    /// because the program owns it.
    fn write_stream_target(
        &mut self,
        object: ObjRef,
        close: bool,
        lines: &[Vec<u8>],
    ) -> Result<(), Failure> {
        for line in lines {
            let value = self.text(line);
            self.roots.push_temp(value);
            let caller = self.caller();
            self.send_message(object, b"LINEOUT", None, &[Some(value)], caller)?;
        }
        if close {
            let caller = self.caller();
            self.send_message(object, b"CLOSE", None, &[], caller)?;
        }
        Ok(())
    }

    /// The qualified name a `STREAM expr` redirection resolves to, echoed as
    /// the keyword result every other target is.
    fn redirect_stream_name(
        &mut self,
        code: &Code<'_>,
        expression: &Expr,
        stream: Stream,
    ) -> Result<Vec<u8>, Failure> {
        let value = self.eval(code, expression)?;
        self.roots.push_temp(value);
        self.trace_redirect_target(stream, value);
        let value = self.required_string_value(value)?;
        let name = self.to_text(value).into_owned();
        let cwd = self.cwd_text();
        Ok(crate::paths::normalize(&String::from_utf8_lossy(&name), &cwd).into_bytes())
    }

    /// A `.Stream` opened for one redirection.
    ///
    /// **The open happens before the command runs**, and its failure is
    /// 98.999 for reading or 98.920 for writing, each carrying the qualified
    /// name and the `ERROR:n` the open answered. Measured: a command whose
    /// output stream cannot be opened never spawns, and a file it would have
    /// written is absent afterwards.
    fn open_named_stream(
        &mut self,
        qualified: &[u8],
        mode: &[u8],
        reading: bool,
    ) -> Result<ObjRef, Failure> {
        let Some(class) = self.rexx_package_class(b"STREAM") else {
            // Only before `StreamClasses.orx` has installed, which is the
            // library bootstrap's own step.
            return Err(Loud::environment_symbol(b".STREAM", "Phase 5").into());
        };
        let argument = self.text(qualified);
        self.roots.push_temp(argument);
        let caller = self.caller();
        let object = self
            .send_message(class, b"NEW", None, &[Some(argument)], caller)?
            .ok_or_else(|| Failure::from(Raised::no_result(b"NEW")))?;
        self.roots.push_temp(object);
        let option = self.text(mode);
        self.roots.push_temp(option);
        let caller = self.caller();
        let answer = self
            .send_message(object, b"OPEN", None, &[Some(option)], caller)?
            .unwrap_or(ObjRef::NIL);
        let answered = self.string_value_text(answer);
        if answered != b"READY:" {
            let raised = match reading {
                true => Raised::stream_not_readable(qualified, &answered),
                false => Raised::stream_not_writeable(qualified, &answered),
            };
            return Err(raised.into());
        }
        Ok(object)
    }

    /// Every line a stream still holds, read as `StreamObjectInputSource`
    /// reads: `LINEIN` until the stream reports `NOTREADY`.
    ///
    /// **The state decides, not the answer.** A blank line answers `''` and so
    /// does the read past the end -- measured over a file of `l1`, `l2`, an
    /// empty line and `l4`, four reads answer with `state` `READY` throughout
    /// and only the fifth is `NOTREADY`, so stopping on an empty answer would
    /// swallow every line after a blank one.
    fn read_stream_lines(&mut self, stream: ObjRef) -> Result<Vec<Vec<u8>>, Failure> {
        let mut lines = Vec::new();
        loop {
            let caller = self.caller();
            let value = self
                .send_message(stream, b"LINEIN", None, &[], caller)?
                .unwrap_or(ObjRef::NIL);
            self.roots.push_temp(value);
            let caller = self.caller();
            let state = self
                .send_message(stream, b"STATE", None, &[], caller)?
                .unwrap_or(ObjRef::NIL);
            if self.string_value_text(state) == b"NOTREADY" {
                return Ok(lines);
            }
            lines.push(self.string_value_text(value));
        }
    }

    /// `StemOutputTarget`: `stem.1` upwards, with `stem.0` rewritten to the
    /// last index used.
    ///
    /// `REPLACE` -- which is also what no option at all means -- drops every
    /// tail and **keeps the stem's own default**, measured: a stem with
    /// `o. = 'dflt'` and `o.7` set answers `dflt` for `o.7` again after one
    /// line of output. `APPEND` starts at `stem.0 + 1`, and a stem with no
    /// `stem.0` of its own is a `REPLACE` however it was asked for -- a
    /// default is not a size count, measured with both `drop p.` and
    /// `q. = 'q'`.
    fn write_stem_target(
        &mut self,
        object: ObjRef,
        append: bool,
        lines: &[Vec<u8>],
    ) -> Result<(), Failure> {
        let start = match append {
            true => self.redirect_stem_count(object)?,
            false => None,
        };
        let mut next = match start {
            Some(count) => count + 1,
            None => {
                self.stem_object_clear(object);
                1
            }
        };
        for line in lines {
            let value = self.text(line);
            let key = next.to_string().into_bytes();
            self.stem_object_write(object, &key, value);
            next += 1;
        }
        let count = self.text((next - 1).to_string().as_bytes());
        self.stem_object_write(object, b"0", count);
        Ok(())
    }

    /// `CollectionOutputTarget`: `EMPTY` unless appending, then one `APPEND`
    /// per line. Both are sends, so the collection's own class decides what
    /// they do -- measured, a `.list~of('keep1','keep2')` given two lines of
    /// output holds those two lines and nothing else, and is still a `List`.
    fn write_collection_target(
        &mut self,
        object: ObjRef,
        append: bool,
        lines: &[Vec<u8>],
    ) -> Result<(), Failure> {
        if !append {
            let caller = self.caller();
            self.send_message(object, b"EMPTY", None, &[], caller)?;
        }
        for line in lines {
            let value = self.text(line);
            self.roots.push_temp(value);
            let caller = self.caller();
            self.send_message(object, b"APPEND", None, &[Some(value)], caller)?;
        }
        Ok(())
    }

    /// `stem.0` as a count, or `None` where the stem carries none of its own.
    ///
    /// **The default does not answer here**, which is what parts this from an
    /// ordinary tail read: measured, `q. = 'q'` then an `APPEND` output
    /// target starts at 1 rather than reporting `q` as a bad count.
    fn redirect_stem_count(&mut self, object: ObjRef) -> Result<Option<usize>, Failure> {
        let Some(value) = self.stem_object_tail(object, b"0") else {
            return Ok(None);
        };
        let text = self.string_value_text(value);
        match whole_number(&text) {
            Some(count) => Ok(Some(count)),
            None => {
                let name = self.stem_object_name(object);
                Err(Raised::stem_size_not_whole(&name, &text).into())
            }
        }
    }

    /// One tail's own value, with no fallback to the stem's default and none
    /// to a derived name. A tombstone answers `None`, as an absent tail does.
    fn stem_object_tail(&self, object: ObjRef, key: &[u8]) -> Option<ObjRef> {
        let Body::Stem { tails, .. } = &self.heap.get(object)?.body else {
            return None;
        };
        match tails.get(key) {
            Some((_, value)) => *value,
            None => None,
        }
    }

    /// One tail as `getFullElement` answers it: its own value, else the
    /// stem's default.
    ///
    /// **A stem with no default answers its own name, with no tail appended**
    /// -- measured, a stem holding `IN.1` and `IN.3` under `IN.0 = 3` feeds a
    /// command `one`, `IN.` and `three`. That is not the `IN.2` an ordinary
    /// `say in.2` derives: this read goes through the stem object, whose
    /// unset value is the name it was created under.
    fn stem_object_read(&mut self, object: ObjRef, key: &[u8]) -> ObjRef {
        let resolved = {
            let Some(found) = self.heap.get(object) else {
                return ObjRef::NIL;
            };
            let Body::Stem { default, tails, .. } = &found.body else {
                return ObjRef::NIL;
            };
            match tails.get(key) {
                Some((_, Some(value))) => Some(*value),
                Some((_, None)) => None,
                None => *default,
            }
        };
        match resolved {
            Some(value) => value,
            None => {
                let name = self.stem_object_name(object);
                self.text(&name)
            }
        }
    }

    /// Writes one tail of a stem object, keeping the insertion ordinal a tail
    /// already has -- the order `allIndexes` answers in depends on it.
    fn stem_object_write(&mut self, object: ObjRef, key: &[u8], value: ObjRef) {
        let Some(found) = self.heap.get_mut(object) else {
            return;
        };
        let Body::Stem { tails, .. } = &mut found.body else {
            return;
        };
        let next = tails.len();
        match tails.get_mut(key) {
            Some((_, existing)) => *existing = Some(value),
            None => {
                tails.insert(key.to_vec(), (next, Some(value)));
            }
        }
    }

    /// Drops every tail **in place**, leaving the stem's default and the
    /// variable's binding alone.
    ///
    /// Not `stem_drop`, which allocates a fresh object and rebinds the
    /// variable for D15a's aliasing rule: here the object is the one the
    /// redirection resolved to and the program's own name for it may not even
    /// be in scope.
    fn stem_object_clear(&mut self, object: ObjRef) {
        let Some(found) = self.heap.get_mut(object) else {
            return;
        };
        let Body::Stem { tails, .. } = &mut found.body else {
            return;
        };
        tails.clear();
    }

    /// A stem object's own name, its trailing period included.
    fn stem_object_name(&self, object: ObjRef) -> Vec<u8> {
        match self.heap.get(object).map(|found| &found.body) {
            Some(Body::Stem { name, .. }) => name.to_vec(),
            _ => Vec::new(),
        }
    }

    /// Whether `value` is a stem object.
    fn is_stem_object(&self, value: ObjRef) -> bool {
        matches!(value.decode(), Decoded::Heap { .. })
            && matches!(
                self.heap.get(value).map(|found| &found.body),
                Some(Body::Stem { .. })
            )
    }

    /// Whether `value` is a `RexxQueue`, which a redirection accepts and this
    /// crate has no queues to give it -- Phase 10's.
    fn is_rexx_queue(&mut self, value: ObjRef) -> bool {
        self.is_instance_of_rexx_class(value, b"REXXQUEUE")
    }

    /// Whether `value` is an object a redirection drives with `LINEIN` and
    /// `LINEOUT`. A `Monitor` counts, which is the oracle's own rule: it
    /// treats one as a stream on both sides.
    fn is_stream_object(&mut self, value: ObjRef) -> bool {
        self.is_instance_of_rexx_class(value, b"INPUTSTREAM")
            || self.is_instance_of_rexx_class(value, b"OUTPUTSTREAM")
            || self.is_instance_of_rexx_class(value, b"MONITOR")
    }

    /// A `.File`'s absolute path, or `None` for anything that is not one. A
    /// `File` redirection is a stream named by that path, not an object driven
    /// directly.
    fn file_object_path(&mut self, value: ObjRef) -> Result<Option<Vec<u8>>, Failure> {
        if !self.is_instance_of_rexx_class(value, b"FILE") {
            return Ok(None);
        }
        let caller = self.caller();
        let answer = self
            .send_message(value, b"ABSOLUTEPATH", None, &[], caller)?
            .unwrap_or(ObjRef::NIL);
        Ok(Some(self.string_value_text(answer)))
    }

    /// Whether `value` is an `OrderedCollection`.
    fn is_ordered_collection(&mut self, value: ObjRef) -> bool {
        self.is_instance_of_rexx_class(value, b"ORDEREDCOLLECTION")
    }

    /// `isInstanceOf` against a class the `REXX` package defines, answering
    /// `false` where that class is not defined at all.
    fn is_instance_of_rexx_class(&mut self, value: ObjRef, upper: &[u8]) -> bool {
        let Some(class) = self.rexx_package_class(upper) else {
            return false;
        };
        let Some(of) = self.class_of_value(value) else {
            return false;
        };
        self.classes().is_a(of, class)
    }
}

/// Whether `value` is a String.
///
/// Every string-valued primitive is one, a whole number in the handle's own
/// tag included -- `receiver_kind` answers `Primitive::String` for a literal
/// and `Primitive::SmallInt` for a small integer, and both take the `String`
/// class. Measured, `with input using 42` feeds one line rather than being
/// asked for an array.
fn is_string(interp: &Interp, value: ObjRef) -> bool {
    match value.decode() {
        Decoded::Text(_) | Decoded::SmallInt(_) => true,
        Decoded::Heap { .. } => matches!(
            interp.heap.get(value).map(|found| &found.body),
            Some(Body::Text { .. } | Body::Num { .. })
        ),
        Decoded::Nil => false,
    }
}

/// `text` as a non-negative whole number, or `None` for anything else --
/// what a `stem.0` has to be.
fn whole_number(text: &[u8]) -> Option<usize> {
    let text = std::str::from_utf8(text).ok()?;
    let trimmed = text.trim_matches(' ');
    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    trimmed.parse().ok()
}
