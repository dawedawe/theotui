use winnow::{
    Parser,
    ascii::multispace0,
    combinator::{delimited, trace},
    error::{ContextError, ErrMode},
};

pub(crate) fn whitespace_wrapped<'i>(
    s: &str,
) -> impl Parser<&'i str, &'i str, ErrMode<ContextError>> {
    trace("whitespace_wrapped", delimited(multispace0, s, multispace0))
}
