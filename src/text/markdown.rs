use markdown::{mdast::Node, ParseOptions};

#[derive(Debug, Clone, PartialEq)]
pub struct RKBMarkdown {
    root: Node,
}

impl RKBMarkdown {}

impl From<String> for RKBMarkdown {
    fn from(value: String) -> Self {
        let options = ParseOptions::gfm();
        let node = markdown::to_mdast(&value, &options).unwrap();
        Self { root: node }
    }
}
