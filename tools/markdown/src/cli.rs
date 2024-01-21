use std::usize;
use std::fs;
use markdown_it;
use markdown_it_footnote;
use rmarkdown:: plugins;

// mod plugins;
// mod common;




fn vistit(level :usize, node: &markdown_it::Node){
    for _ in 0..level{
        print!("    ");
    }
    println!("{:?}", node.node_type);
    for child in &node.children{
        vistit(level+1, child)
    }
}


fn main() {
    let parser: &mut markdown_it::MarkdownIt = &mut markdown_it::MarkdownIt::new();
    
    markdown_it::plugins::sourcepos::add(parser);
    markdown_it::plugins::cmark::add(parser);
    markdown_it_footnote::add(parser);

    plugins::inline::add(parser);
 

    let content = fs::read_to_string(r"E:\docs\obsidian_orange\Books\个人成长\即兴演讲：掌控人生关键时刻.md")
    .expect("读文件失败");

    // plugins::unicode_codecs::add(parser);
    let actual = parser.parse(&content.as_str()).render();


    println!("{}", plugins::unicode_codecs::unicode2dec(actual));

    let nodes2 = parser.parse("**foo**, ==gg==");
    vistit(0, &nodes2);    
    
}