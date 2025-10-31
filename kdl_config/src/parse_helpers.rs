use crate::error::ParseDiagnostic;
use kdl::{KdlDocument, KdlNode, KdlValue};
use miette::NamedSource;

pub fn get_single_argument_value<'a>(
    input: NamedSource<String>,
    node: &'a KdlNode,
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> Option<&'a KdlValue> {
    let entry_len = node.entries().len();
    if node.entries().len() != 1 {
        let extra_entries: Vec<String> = node
            .entries()
            .iter()
            .skip(1)
            // TODO: disallow named values
            .map(|x| x.value().to_string())
            .collect();
        diagnostics.push(ParseDiagnostic {
            input: input.clone(),
            span: node.span(),
            message: Some(format!(
                "Node should only contain 1 entry but contained {entry_len:?}"
            )),
            label: None,
            help: Some(format!(
                "Consider removing the extra entries {extra_entries:?}",
            )),
            severity: miette::Severity::Error,
        });
    }
    Some(node.entries().first().unwrap().value())
}

pub fn get_children<'a, const N: usize>(
    input: NamedSource<String>,
    node: &'a KdlNode,
    names: [&str; N],
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> [Option<&'a KdlNode>; N] {
    match node.children() {
        Some(children) => get_children_of_document(input, children, names, diagnostics),
        None => {
            diagnostics.push(ParseDiagnostic {
                input: input.clone(),
                span: node.span(),
                message: Some(format!(
                    "Node has no children but expected children with names {names:?}"
                )),
                label: None,
                help: None,
                severity: miette::Severity::Error,
            });
            [None; N]
        }
    }
}

pub fn get_children_of_document<'a, const N: usize>(
    input: NamedSource<String>,
    children: &'a KdlDocument,
    names: [&str; N],
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> [Option<&'a KdlNode>; N] {
    let mut result_children = vec![];
    let mut missing_fields = vec![];
    for name in names {
        if let Some(child) = children.get(name) {
            result_children.push(Some(child))
        } else {
            result_children.push(None);
            diagnostics.push(ParseDiagnostic {
                input: input.clone(),
                span: children.span(),
                message: Some(format!("Child {name} is missing from this node")),
                label: None,
                help: None,
                severity: miette::Severity::Error,
            });
            missing_fields.push(name);
        }
    }

    for child in children.nodes() {
        if !names.contains(&child.name().value()) {
            diagnostics.push(ParseDiagnostic {
                input: input.clone(),
                span: child.span(),
                message: Some("Unknown node name".to_owned()),
                label: None,
                help: Some(if missing_fields.is_empty() {
                    "This node already has all the children it needs. Consider removing this section.".to_owned()
                } else {
                    format!("Consider one of these {names:?} instead?")
                }),
                severity: miette::Severity::Error,
            });
        }
    }
    result_children.try_into().unwrap()
}
