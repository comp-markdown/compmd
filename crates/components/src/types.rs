//! The types TypeScript states, as far as a signature's type can be checked against them, and the props a type lists.

use compmd::analysis::{Prop, PropType};
use oxc_ast::ast::{
    Declaration, Program, PropertyKey, Statement, TSInterfaceDeclaration, TSLiteral, TSSignature,
    TSType, TSTypeAliasDeclaration, TSTypeName,
};

/// The type names that are a slot's: the node or the snippet a binding gives for one (spec §7.6).
const SLOT_TYPES: [&str; 4] = ["ReactNode", "ReactElement", "JSX.Element", "Snippet"];

/// A type name as written: `ReactNode`, `React.ReactNode`, `JSX.Element`.
fn name_of(name: &TSTypeName<'_>) -> Option<String> {
    match name {
        TSTypeName::IdentifierReference(identifier) => Some(identifier.name.to_string()),
        TSTypeName::QualifiedName(qualified) => Some(format!(
            "{}.{}",
            name_of(&qualified.left)?,
            qualified.right.name
        )),
        TSTypeName::ThisExpression(_) => None,
    }
}

/// Whether `member` of a union adds no value a prop can be given: `undefined` or `null`.
fn is_nothing(member: &TSType<'_>) -> bool {
    matches!(
        member,
        TSType::TSUndefinedKeyword(_) | TSType::TSNullKeyword(_)
    )
}

/// The literal `member` is, when it is one.
fn literal_of<'t, 'a>(member: &'t TSType<'a>) -> Option<&'t TSLiteral<'a>> {
    match member {
        TSType::TSLiteralType(literal) => Some(&literal.literal),
        _ => None,
    }
}

/// The type `node` states, when it is one a signature's type can be checked against.
pub(crate) fn type_of(node: &TSType<'_>) -> Option<PropType> {
    match node {
        TSType::TSStringKeyword(_) => Some(PropType::String),
        TSType::TSNumberKeyword(_) => Some(PropType::Number),
        TSType::TSBooleanKeyword(_) => Some(PropType::Boolean),
        TSType::TSParenthesizedType(inner) => type_of(&inner.type_annotation),
        TSType::TSTypeReference(reference) => {
            let name = name_of(&reference.type_name)?;
            let name = name.strip_prefix("React.").unwrap_or(&name);
            SLOT_TYPES.contains(&name).then_some(PropType::Slot)
        }
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(string) => Some(PropType::Enum {
                members: vec![string.value.to_string()],
            }),
            _ => None,
        },
        TSType::TSUnionType(union) => {
            let members: Vec<&TSType<'_>> = union
                .types
                .iter()
                .filter(|member| !is_nothing(member))
                .collect();
            if let [only] = members.as_slice() {
                return type_of(only);
            }
            let literals: Option<Vec<&TSLiteral<'_>>> =
                members.iter().map(|member| literal_of(member)).collect();
            let literals = literals?;
            let strings: Option<Vec<String>> = literals
                .iter()
                .map(|literal| match literal {
                    TSLiteral::StringLiteral(string) => Some(string.value.to_string()),
                    _ => None,
                })
                .collect();
            if let Some(members) = strings {
                return Some(PropType::Enum { members });
            }
            literals
                .iter()
                .all(|literal| matches!(literal, TSLiteral::BooleanLiteral(_)))
                .then_some(PropType::Boolean)
        }
        _ => None,
    }
}

/// The name of a property key a source writes as a name or a string, never computed.
pub(crate) fn key_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(string) => Some(string.value.to_string()),
        _ => None,
    }
}

/// The props of a type's members: its properties, by name; `None` when one isn't a property with a plain key.
fn props_of_members(members: &[TSSignature<'_>]) -> Option<Vec<Prop>> {
    members
        .iter()
        .map(|member| match member {
            TSSignature::TSPropertySignature(property) if !property.computed => Some(Prop {
                name: key_name(&property.key)?,
                kind: property
                    .type_annotation
                    .as_ref()
                    .and_then(|annotation| type_of(&annotation.type_annotation)),
                optional: property.optional,
                default: None,
            }),
            _ => None,
        })
        .collect()
}

/// The members of the interface or type alias `name` declares at the top level of `program`, exported or not, as
/// props.
fn named_members(program: &Program<'_>, name: &str) -> Option<Vec<Prop>> {
    for statement in &program.body {
        let declaration = match statement {
            Statement::TSInterfaceDeclaration(interface) => Some(Declared::Interface(interface)),
            Statement::TSTypeAliasDeclaration(alias) => Some(Declared::Alias(alias)),
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSInterfaceDeclaration(interface)) => {
                    Some(Declared::Interface(interface))
                }
                Some(Declaration::TSTypeAliasDeclaration(alias)) => Some(Declared::Alias(alias)),
                _ => None,
            },
            _ => None,
        };
        match declaration {
            Some(Declared::Interface(interface)) if interface.id.name == name => {
                return (interface.extends.is_empty() && interface.type_parameters.is_none())
                    .then(|| props_of_members(&interface.body.body))
                    .flatten();
            }
            Some(Declared::Alias(alias)) if alias.id.name == name => {
                return alias
                    .type_parameters
                    .is_none()
                    .then(|| members_of(program, &alias.type_annotation))
                    .flatten();
            }
            _ => {}
        }
    }
    None
}

/// A type a file declares, which props may be named by.
enum Declared<'d, 'a> {
    Interface(&'d TSInterfaceDeclaration<'a>),
    Alias(&'d TSTypeAliasDeclaration<'a>),
}

/// The props a type lists: a type literal, an interface or a type alias of the file, or an intersection of them.
pub(crate) fn members_of(program: &Program<'_>, node: &TSType<'_>) -> Option<Vec<Prop>> {
    match node {
        TSType::TSTypeLiteral(literal) => props_of_members(&literal.members),
        TSType::TSParenthesizedType(inner) => members_of(program, &inner.type_annotation),
        TSType::TSIntersectionType(intersection) => {
            let parts: Option<Vec<Vec<Prop>>> = intersection
                .types
                .iter()
                .map(|part| members_of(program, part))
                .collect();
            parts.map(|parts| parts.into_iter().flatten().collect())
        }
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            match &reference.type_name {
                TSTypeName::IdentifierReference(identifier) => {
                    named_members(program, &identifier.name)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
