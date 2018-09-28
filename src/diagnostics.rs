use srcmap::{BytePos, Span};

#[derive(Debug, PartialEq)]
pub enum ReportCode {
    MissingExponentDigits {
        exp_pos: BytePos,
    },
    MissingTerminatingStringMark {
        str_start_pos: BytePos,
        eol_pos: BytePos,
    },
    UnknownCharacter {
        pos: BytePos,
    },
    FatalError,
}

pub struct Diagnostic {
    code: ReportCode,
    spans: Vec<Span>,
}

pub struct DiagnosticBuilder<'a> {
    handler: &'a Handler,
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

    fn span(mut self, s: Span) -> DiagnosticBuilder<'a> {
        self.diagnostic.spans.push(s);
        self
    }

    pub fn emit(self) -> bool {
        (self.handler.emitter)(self.diagnostic)
    }
}

pub struct Handler {
    emitter: Box<Fn(Diagnostic) -> bool>,
}

impl Handler {
    pub fn with_emitter<E>(emitter: E) -> Handler
    where
        E: Fn(Diagnostic) -> bool + 'static,
    {
        Handler {
            emitter: Box::new(emitter),
        }
    }

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
