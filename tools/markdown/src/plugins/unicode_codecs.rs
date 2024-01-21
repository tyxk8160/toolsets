use markdown_it::parser::inline::Text;
use markdown_it::{self, MarkdownIt, Node};

use markdown_it::parser::block::builtin::BlockParserRule;
use markdown_it::parser::core::CoreRule;
use markdown_it::parser::inline::builtin::InlineParserRule;

/// Add the full footnote plugin to the parser
pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<UnicodeHtml>()
        .after::<BlockParserRule>()
        .after::<InlineParserRule>();
}

pub struct UnicodeHtml;

impl CoreRule for UnicodeHtml {
    fn run(root: &mut Node, _: &MarkdownIt) {
        root.walk_mut(|node: &mut Node, _| {
            if let Some(node_ref) = node.cast_mut::<Text>() {
                let mut s: String = "".into();
                for c in node_ref.content.as_str().chars() {
                    let d = c as u32;

                    s = if d > 128 {
                        format!("{}&#{};", s, d)
                    } else {
                        format!("{}{}", s, c.to_string())
                    };
                }
                node_ref.content=s;
            }

            println!("node:{:?}", node.node_value);
        });
    }
}



pub fn unicode2dec(old:String)->String{

    let mut s: String = "".into();
    for c in old.as_str().chars() {
        let d = c as u32;

        s = if d > 128 {
            format!("{}&#{};", s, d)
        } else {
            format!("{}{}", s, c.to_string())
        };
    }
    s


}