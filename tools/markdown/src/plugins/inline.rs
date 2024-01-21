use markdown_it::generics::inline::emph_pair;
use markdown_it::Node;

use markdown_it::{self, parser::inline::InlineRule, MarkdownIt, NodeValue};
use path_absolutize::*; // for path normalize
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::{char, fs, path};

use crate::common::error::RMarkodwnError;

///
/// 1. 语法高亮mark, 语法
/// ```markdown
/// ==text==
/// ```
/// 输出
/// ```html
/// <mark> text </mark>
/// ```
/// 2. 上标语法
/// ```markdown
/// a^2^
/// ```
/// 输出
/// ```html
/// <p>a <sup>2</sup>
/// ````
/// 3. 添加obsidian 双链接解析语法
/// ```markdown
/// md文件链接: [[文件路径|文件标题]]
/// md图片链接:![[xxx.paste|xxx]]
/// ```
/// 输出
/// ```html
/// md文件链接: <a href="文件路径.html"> 文件标题</a>
/// md图片链接：<img embed="" src="xx.png" />
/// ````
pub fn add(md: &mut MarkdownIt) {
    emph_pair::add_with::<'=', 2, true>(md, || Node::new(MarkNode));
    emph_pair::add_with::<'^', 1, false>(md, || Node::new(SupNode));

    md.inline.add_rule::<ObsidianLinkRule<'!'>>();
    md.inline.add_rule::<ObsidianLinkRule<'['>>();

}

#[derive(Debug)]
pub struct MarkNode;
impl NodeValue for MarkNode {
    fn render(&self, node: &markdown_it::Node, fmt: &mut dyn markdown_it::Renderer) {
        let attrs = node.attrs.clone();
        fmt.open("mark", &attrs);
        fmt.contents(&node.children);
        fmt.close("mark");
    }
}

#[derive(Debug)]
pub struct SupNode;
impl NodeValue for SupNode {
    fn render(&self, node: &Node, fmt: &mut dyn markdown_it::Renderer) {
        fmt.open("sup", &node.attrs);
        fmt.contents(&node.children);
        fmt.close("sup");
    }
}

///
/// obsidian通用链接形式, [[filename#head^label|aliases]]
/// 根据文件类型选择适合的渲染
#[derive(Debug)]
pub struct ObsidianLinkNode {
    pub filename: String,
    pub head: Option<String>,
    pub label: Option<String>,
    pub aliases: Option<String>,
    pub embed: bool,
    pub image_suffix: Option<String>,
}

fn image_process(
    basedir: &Path,
    dstdir: &Path,
    filename: &str,
    image_suffix: &Option<String>,
) -> Result<String, RMarkodwnError> {
    let suffix = match image_suffix {
        Some(x) => Ok(x),
        None => Err(RMarkodwnError::Unknown("suffix is not image".to_string())),
    }?;

    let src_image_path = &basedir.join(filename).absolutize()?.to_path_buf();

    let meta = fs::metadata(src_image_path)?;
    let s = meta.modified()?.duration_since(UNIX_EPOCH)?.as_secs();
    let new_image_basename = format!("{}.{}", s, suffix);
    let dst_image_path = &dstdir.join(&new_image_basename).absolutize()?.to_path_buf();
    if fs::metadata(&dst_image_path).is_ok() {
        // todo: 需要拷贝文件到supermemo中，以后完成
        println!("{:?} is exists", dst_image_path);
    } else {
        println!("copy from {:? } to {:?}", src_image_path, dst_image_path);
    }
    Ok(new_image_basename)
}

impl ObsidianLinkNode {
    fn render_image(&self, node: &Node, fmt: &mut dyn markdown_it::Renderer) {
        // 配置
        let base_dir = path::Path::new("E:/docs/obsidian_orange/101-SlipBox/01-FleetingBox/xx.md")
            .parent()
            .unwrap();

        let dst_dir = path::Path::new("d:/sm18-lazy-package-1.2.2/sm18/systems/gg/elements/");
        let href_base_dir = "d:/sm18-lazy-package-1.2.2/sm18/systems/gg/elements/";

        let result = image_process(base_dir, dst_dir, &self.filename, &self.image_suffix);

        let href = match result {
            Ok(x) => {
                format!("file:///{}{}", href_base_dir, x)
            }
            Err(err) => {
                println!("error is:{:?}", err);
                format!("file:///{}{}", href_base_dir, self.filename)
            }
        };

        // "![[../../99-attach/Pasted image 20230712231411.png]]"
        // E:/docs/obsidian_orange/101-SlipBox/01-FleetingBox

        let mut attrs = node.attrs.clone();
        if self.embed {
            attrs.push(("embed", "".into()));
        }

        if let Some(aliases) = &self.aliases {
            attrs.push(("size", aliases.clone()));
        }

        attrs.push(("alt", self.filename.clone()));
        attrs.push(("src", href));
        fmt.open("img", &attrs);
        fmt.close("img")
    }

    fn render_link(&self, node: &Node, fmt: &mut dyn markdown_it::Renderer) {
        let filename = &self.filename;
        let mut href: String = format!("{}.html", filename);
        if let Some(head) = self.head.as_ref() {
            href = format!("{}#{}", href, head);
        }
        let title = if let Some(aliases) = self.aliases.as_ref() {
            format!("{}", aliases)
        } else {
            format!("{}", filename)
        };
        let mut attrs = node.attrs.clone();

        attrs.push(("href", href));
        fmt.open("a", &attrs);
        fmt.text(title.as_str());
        fmt.close("a");
    }
}

impl NodeValue for ObsidianLinkNode {
    fn render(&self, node: &Node, fmt: &mut dyn markdown_it::Renderer) {
        if self.image_suffix.is_none() {
            self.render_link(node, fmt);
        } else {
            self.render_image(node, fmt);
        }
    }
}

pub struct ObsidianLinkRule<const PREFIX: char>;
impl<const PREFIX: char> InlineRule for ObsidianLinkRule<PREFIX> {
    const MARKER: char = PREFIX;
    fn check(state: &mut markdown_it::parser::inline::InlineState) -> Option<usize> {
        let mut chars = state.src[state.pos..state.pos_max].chars();
        if PREFIX == '!' {
            if chars.next() != Some(PREFIX) {
                return None;
            }
        }

        if chars.next() != Some('[') {
            return None;
        }
        if chars.next() != Some('[') {
            return None;
        }

        return Some(3);
    }

    fn run(state: &mut markdown_it::parser::inline::InlineState) -> Option<(Node, usize)> {
        let pos = state.pos;
        let pos_max = state.pos_max;

        let mut chars = state.src[state.pos..state.pos_max].chars();
        let mut offset: usize = 0;
        let mut embed = false;
        if PREFIX == '!' {
            let value = chars.next();
            if value != Some('!') {
                return None;
            }
            offset = 1;
            embed = true;
        }

        if chars.next() != Some('[') {
            return None;
        }
        if chars.next() != Some('[') {
            return None;
        }

        let input = &state.src[(pos + 2 + offset)..pos_max];

        let left_bracket = input.find("[[");
        let right_bracket = input.find("]]");
        if right_bracket.is_none() {
            return None;
        }
        let right_pos = right_bracket.unwrap();
        if left_bracket.is_some() && left_bracket.unwrap() < right_pos {
            return None;
        }

        let ob_link = &input[0..right_pos];
        if let Some(ob) = parse_obsidian_link(ob_link, embed) {
            let node = Node::new(ob);
            let lenght = right_pos + 4 + offset; // [[ 移动了四格，
            return Some((node, lenght));
        }

        None
    }
}

struct MarkTextRule;

impl InlineRule for MarkTextRule {
    const MARKER: char = '=';

    fn check(_: &mut markdown_it::parser::inline::InlineState) -> Option<usize> {
        None
    }

    fn run(
        state: &mut markdown_it::parser::inline::InlineState,
    ) -> Option<(markdown_it::Node, usize)> {
        let mut chars = state.src[state.pos..state.pos_max].chars();

        if chars.next().unwrap() != Self::MARKER {
            return None;
        }

        let scanned = state.scan_delims(state.pos, true);
        println!("hh:{:?}, {:?}", scanned, state.pos);
        let end_pos = state.pos + scanned.length;

        println!("mark:{:?}", state.src[state.pos..end_pos].chars());
        let node = Node::new(MarkNode);

        println!("{:?}", node);
        Some((node, scanned.length))
    }
}

///
/// 处理<filename>#<head>^<label>|<aliases>
pub fn parse_obsidian_link(input: &str, embed: bool) -> Option<ObsidianLinkNode> {
    let mut filename_option: Option<String> = None;
    let mut head: Option<String> = None;
    let mut label: Option<String> = None;
    let mut aliases: Option<String> = None;

    let mut prev_status: usize = 0; // 记录状态。 0：filename 1：head 2: label 3:aliases
    let mut prev: usize = 0; // 记录索引
    let mut chars = input.char_indices();

    while let Some((index, c)) = chars.next() {
        let new_status: usize;
        match c {
            '#' => new_status = 1,
            '^' => new_status = 2,
            '|' => new_status = 3,

            _ => continue,
        }
        let value: Option<String> = Some(String::from(&input[prev..index]));
        match prev_status {
            0 => filename_option = value,
            1 => head = value,
            2 => label = value,
            _ => {}
        }
        prev = index + 1;
        prev_status = new_status;
        if prev_status == 3 {
            break;
        }
    }

    let value = if prev < input.len() {
        Some(String::from(&input[prev..input.len()]))
    } else {
        None
    };

    match prev_status {
        0 => filename_option = value,
        1 => head = value,
        2 => label = value,
        3 => aliases = value,
        _ => {}
    };
    if filename_option.is_none() {
        return None;
    }
    // 判断图片逻辑
    let filename = filename_option.unwrap();
    let mut image_suffix: Option<String> = None;
    if let Some((_, suffix)) = filename.rsplit_once(".") {
        image_suffix = match suffix.to_lowercase().as_str() {
            "png" | "jpeg" | "jpg" | "gif" => Some(String::from(suffix)),
            _ => None,
        }
    }

    let node = ObsidianLinkNode {
        filename,
        head,
        label,
        aliases,
        embed,
        image_suffix,
    };
    Some(node)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_mark() {
        use markdown_it::MarkdownIt;
        let md = &mut MarkdownIt::new();
        add(md);

        let input = "![[../../99-attach/Pasted image 20230712231411.png]]";

        let html = md.parse(input).render();
        print!("{}", html)
    }
}
