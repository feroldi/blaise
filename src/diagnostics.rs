use srcmap::{BytePos, Span};

/// A `ReportCode` value gathers enough information about some error in the
/// parsing process. It is used by the diagnostics system to report good
/// quality error messages.
#[derive(Debug, PartialEq)]
pub enum ReportCode {
    /// Numeric literals with no digits after an exponent.
    MissingExponentDigits { exp_pos: BytePos },
    /// String literals missing a terminating quotation mark.
    MissingTerminatingStringMark {
        str_start_pos: BytePos,
        eol_pos: BytePos,
    },
    /// Unknown character in the source code.
    UnknownCharacter { pos: BytePos },
    /// Error which can't let the compiling process continue.
    FatalError,
}

/// A `Diagnostic` holds a report code, and a list of code snippets related to
/// the error, which is useful for contextualization when reporting diagnostics.
pub struct Diagnostic {
    /// The code report with context about the diagnostic.
    code: ReportCode,
    /// Code snippets that are related to the diagnostic and useful for
    /// the programmer.
    spans: Vec<Span>,
}

/// A helper to construct a diagnostic when reporting.
pub struct DiagnosticBuilder<'a> {
    /// The associated diagnostic handler for emission.
    handler: &'a Handler,
    /// The diagnostic being built.
    diagnostic: Diagnostic,
}

impl<'a> DiagnosticBuilder<'a> {
    fn with_diagnostic(
        handler: &'a Handler,
        diagnostic: Diagnostic,
    ) -> DiagnosticBuilder<'a> {
        DiagnosticBuilder {
            handler,
            diagnostic,
        }
    }

    /// Adds a span to the diagnostic.
    pub fn span(mut self, s: Span) -> DiagnosticBuilder<'a> {
        self.diagnostic.spans.push(s);
        self
    }

    /// Emits the diagnostic.
    pub fn emit(self) -> bool {
        (self.handler.emitter)(self.diagnostic)
    }
}

/// The diagnostics handler. This handler takes care of emitting diagnostics to
/// e.g. the user.
pub struct Handler {
    /// The emitter callback.
    emitter: Box<Fn(Diagnostic) -> bool>,
}

impl Handler {
    /// Creates a `Handler` with an emitter callback.
    pub fn with_emitter<E>(emitter: E) -> Handler
    where
        E: Fn(Diagnostic) -> bool + 'static,
    {
        Handler {
            emitter: Box::new(emitter),
        }
    }

    /// Constructs a diagnostic builder with a report code. This is used to
    /// help reporting errors, and is the main method of diagnostic reporting.
    pub fn report<'a>(&'a self, code: ReportCode) -> DiagnosticBuilder<'a> {
        let diagnostic = Diagnostic {
            code,
            spans: vec![],
        };
        DiagnosticBuilder::with_diagnostic(self, diagnostic)
    }
}

#[cfg(test)]
mod test {
    use super::{Diagnostic, Handler, ReportCode};
    use srcmap::{BytePos, Span};

    #[test]
    fn diagnostic_builder_test() {
        let handler = Handler::with_emitter(|diag: Diagnostic| {
            assert_eq!(ReportCode::FatalError, diag.code);
            assert_eq!(2, diag.spans.len());
            assert_eq!(
                Span {
                    start: BytePos(0),
                    end: BytePos(5),
                },
                diag.spans[0]
            );
            assert_eq!(
                Span {
                    start: BytePos(6),
                    end: BytePos(10),
                },
                diag.spans[1]
            );
            true
        });

        let result = handler
            .report(ReportCode::FatalError)
            .span(Span {
                start: BytePos(0),
                end: BytePos(5),
            })
            .span(Span {
                start: BytePos(6),
                end: BytePos(10),
            })
            .emit();

        assert!(result);
    }
}
