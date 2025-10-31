use error::{ParseDiagnostic, ParseError};
use kdl::{KdlDocument, KdlNode};
use miette::{NamedSource, SourceOffset, SourceSpan};

pub mod arrayvec;
pub mod error;
pub mod integers;
pub mod parse_helpers;

pub fn parse<T: KdlConfig>(
    input: NamedSource<String>,
    document: KdlDocument,
) -> (Parsed<T>, ParseError) {
    let mut diagnostics = vec![];

    // Construct a fake node since we start with a document but need a node.
    let mut fake_node = KdlNode::new("");
    fake_node.set_children(document);

    (
        KdlConfig::parse_as_node(input.clone(), &fake_node, &mut diagnostics),
        ParseError { input, diagnostics },
    )
}

/// manually implement for now, derive this later
pub trait KdlConfig {
    fn parse_as_node(
        source: NamedSource<String>,
        node: &KdlNode,
        diagnostics: &mut Vec<ParseDiagnostic>,
    ) -> Parsed<Self>
    where
        Self: Sized;
}

/// Convert the KdlConfig structure into a finalized struct.
/// The #[Derive(KdlConfigFinalize)] assumes that the finalize type has the exact same structure with the `Parsed` wrappers removed.
/// If your final structure differs from this you can manually implement KdlConfigFinalize for your type.
/// Or just completely ignore `KdlConfigFinalize`, it is ultimately an optional convenience on top of KdlConfig.
pub trait KdlConfigFinalize {
    type FinalizeType;
    fn finalize(&self) -> Self::FinalizeType;
}

pub struct Parsed<T> {
    /// The actual parsed value
    pub value: T,
    /// The span of the entire KDL node
    pub full_span: SourceSpan,
    /// The span of the KDL nodes identifier
    pub name_span: SourceSpan,
    /// When a field cannot be parsed, this field is set to `false` and `value` is set to `Default::default`
    pub valid: bool,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Parsed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parsed")
            .field("value", &self.value)
            .field("valid", &self.valid)
            .finish()
    }
}

impl<T: Default> Default for Parsed<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            full_span: SourceSpan::new(SourceOffset::from_location("", 0, 0), 0),
            name_span: SourceSpan::new(SourceOffset::from_location("", 0, 0), 0),
            valid: Default::default(),
        }
    }
}
