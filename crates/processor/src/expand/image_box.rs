//! An image box (spec §9.2): a box whose content in the expanded tree is one paragraph holding one image, alone or as
//! a link's only content, white space aside. Its image takes the box's inside, as `object-fit` and `object-position`
//! say.

use crate::expanded::{Content, Node};

/// Whether a box holding `children` is an image box.
pub(super) fn is_image_box(children: &[Node]) -> bool {
    let [paragraph] = children else {
        return false;
    };
    let Content::Paragraph(paragraph) = &paragraph.content else {
        return false;
    };
    is_one_image(&paragraph.children)
}

/// Whether `inlines` are one image, or one link holding one image, white space aside.
fn is_one_image(inlines: &[Node]) -> bool {
    let mut shown = inlines.iter().filter(|node| !is_white_space(node));
    let (Some(only), None) = (shown.next(), shown.next()) else {
        return false;
    };
    match &only.content {
        Content::Image(_) | Content::ImageReference(_) => true,
        Content::Link(link) => is_one_image(&link.children),
        Content::LinkReference(link) => is_one_image(&link.children),
        _ => false,
    }
}

fn is_white_space(node: &Node) -> bool {
    matches!(&node.content, Content::Text(text) if text.value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Image, Link, Literal, Parent};

    fn node(content: Content) -> Node {
        Node {
            content,
            origin: None,
        }
    }

    fn text(value: &str) -> Node {
        node(Content::Text(Literal {
            value: value.into(),
            ..Literal::default()
        }))
    }

    fn image() -> Node {
        node(Content::Image(Image {
            url: "harbour.jpg".into(),
            alt: "Boats".into(),
            ..Image::default()
        }))
    }

    fn link(children: Vec<Node>) -> Node {
        node(Content::Link(Link {
            url: "harbour.comp.md".into(),
            children,
            title: None,
            url_parts: None,
            pack: false,
            position: None,
        }))
    }

    fn paragraph(children: Vec<Node>) -> Node {
        node(Content::Paragraph(Parent {
            children,
            position: None,
        }))
    }

    #[test]
    fn one_image_alone_or_linked_is_an_image_box() {
        assert!(is_image_box(&[paragraph(vec![image()])]));
        assert!(is_image_box(&[paragraph(vec![
            text(" "),
            image(),
            text("\n")
        ])]));
        assert!(is_image_box(&[paragraph(vec![link(vec![image()])])]));
    }

    #[test]
    fn anything_beside_the_image_is_no_image_box() {
        assert!(!is_image_box(&[]));
        assert!(!is_image_box(&[paragraph(vec![image(), image()])]));
        assert!(!is_image_box(&[paragraph(vec![image(), text(" Boats")])]));
        assert!(!is_image_box(&[paragraph(vec![link(vec![
            image(),
            text("Boats")
        ])])]));
        assert!(!is_image_box(&[paragraph(vec![text("Boats")])]));
        assert!(!is_image_box(&[
            paragraph(vec![image()]),
            paragraph(vec![text("A caption")])
        ]));
        assert!(!is_image_box(&[text("loose")]));
    }
}
