use markdown::{mdast::Node, ParseOptions};

#[derive(Debug, Clone, PartialEq)]
pub struct RKBMarkdown {
    node: Node,
}

impl RKBMarkdown {
    pub fn to_string(&self) -> String {
        self.node.to_string()
    }
}

impl From<String> for RKBMarkdown {
    fn from(value: String) -> Self {
        let options = ParseOptions::gfm();
        let node = markdown::to_mdast(&value, &options).unwrap();
        Self { node }
    }
}
