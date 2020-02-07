use std::fmt::Debug;

use super::func::FuncHeader;
use super::expr::{Expr, Tuple, Object};
use super::span::{Span, Spanned};
use super::tokens::Token;
use super::*;


/// Check whether the expected and found results for the given source code
/// match by the comparison function, and print them out otherwise.
pub fn check<T>(src: &str, exp: T, found: T, spans: bool)
where T: Debug + PartialEq + SpanlessEq {
    let cmp = if spans { PartialEq::eq } else { SpanlessEq::spanless_eq };
    if !cmp(&exp, &found) {
        println!("source:   {:?}", src);
        println!("expected: {:#?}", exp);
        println!("found:    {:#?}", found);
        panic!("test failed");
    }
}

/// Create a vector of optionally spanned expressions from a list description.
///
/// # Examples
/// When you want to add span information to the items, the format is as
/// follows.
/// ```
/// spanned![(0:0, 0:5, "hello"), (0:5, 0:3, "world")]
/// ```
/// The span information can simply be omitted to create a vector with items
/// that are spanned with dummy zero spans.
macro_rules! spanned {
    (item ($sl:tt:$sc:tt, $el:tt:$ec:tt, $v:expr)) => ({
        #[allow(unused_imports)]
        use $crate::syntax::span::{Position, Span, Spanned};
        Spanned {
            span: Span::new(
                Position::new($sl, $sc),
                Position::new($el, $ec)
            ),
            v: $v
        }
    });

    (vec $(($sl:tt:$sc:tt, $el:tt:$ec:tt, $v:expr)),* $(,)?) => {
        (vec![$(spanned![item ($sl:$sc, $el:$ec, $v)]),*], true)
    };

    (vec $($v:expr),* $(,)?) => {
        (vec![$($crate::syntax::test::zspan($v)),*], false)
    };
}

/// Span an element with a zero span.
pub fn zspan<T>(v: T) -> Spanned<T> {
    Spanned { v, span: Span::ZERO }
}

function! {
    /// Most functions in the tests are parsed into the debug function for easy
    /// inspection of arguments and body.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DebugFn {
        pub header: FuncHeader,
        pub body: Option<SyntaxModel>,
    }

    parse(header, body, ctx, f) {
        let cloned = header.clone();
        header.args.pos.items.clear();
        header.args.key.pairs.clear();
        DebugFn {
            header: cloned,
            body: body!(opt: body, ctx, f),
        }
    }

    layout(self, ctx, errors) { vec![] }
}

/// Compares elements by only looking at values and ignoring spans.
pub trait SpanlessEq<Rhs=Self> {
    fn spanless_eq(&self, other: &Rhs) -> bool;
}

impl<T: SpanlessEq> SpanlessEq for Vec<Spanned<T>> {
    fn spanless_eq(&self, other: &Vec<Spanned<T>>) -> bool {
        self.len() == other.len()
        && self.iter().zip(other).all(|(x, y)| x.v.spanless_eq(&y.v))
    }
}

impl SpanlessEq for SyntaxModel {
    fn spanless_eq(&self, other: &SyntaxModel) -> bool {
        self.nodes.spanless_eq(&other.nodes)
    }
}

impl SpanlessEq for Node {
    fn spanless_eq(&self, other: &Node) -> bool {
        fn downcast<'a>(func: &'a (dyn Model + 'static)) -> &'a DebugFn {
            func.downcast::<DebugFn>().expect("not a debug fn")
        }

        match (self, other) {
            (Node::Model(a), Node::Model(b)) => {
                downcast(a.as_ref()).spanless_eq(downcast(b.as_ref()))
            }
            (a, b) => a == b,
        }
    }
}

impl SpanlessEq for DebugFn {
    fn spanless_eq(&self, other: &DebugFn) -> bool {
        self.header.name.v == other.header.name.v
        && self.header.args.pos.spanless_eq(&other.header.args.pos)
        && self.header.args.key.spanless_eq(&other.header.args.key)
    }
}

impl SpanlessEq for Expr {
    fn spanless_eq(&self, other: &Expr) -> bool {
        match (self, other) {
            (Expr::Tuple(a), Expr::Tuple(b)) => a.spanless_eq(b),
            (Expr::Object(a), Expr::Object(b)) => a.spanless_eq(b),
            (a, b) => a == b,
        }
    }
}

impl SpanlessEq for Tuple {
    fn spanless_eq(&self, other: &Tuple) -> bool {
        self.items.len() == other.items.len()
        && self.items.iter().zip(&other.items)
            .all(|(x, y)| x.v.spanless_eq(&y.v))
    }
}

impl SpanlessEq for Object {
    fn spanless_eq(&self, other: &Object) -> bool {
        self.pairs.len() == other.pairs.len()
        && self.pairs.iter().zip(&other.pairs)
            .all(|(x, y)| x.key.v == y.key.v && x.value.v.spanless_eq(&y.value.v))
    }
}

/// Implement `SpanlessEq` by just forwarding to `PartialEq`.
macro_rules! forward {
    ($type:ty) => {
        impl SpanlessEq for $type {
            fn spanless_eq(&self, other: &$type) -> bool {
                self == other
            }
        }
    };
}

forward!(String);
forward!(Token<'_>);
forward!(Decoration);
