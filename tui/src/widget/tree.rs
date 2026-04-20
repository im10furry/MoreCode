#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }
}

pub fn render_tree(nodes: &[TreeNode]) -> String {
    let mut lines = Vec::new();
    for node in nodes {
        render_node(node, 0, &mut lines);
    }
    lines.join("\n")
}

fn render_node(node: &TreeNode, depth: usize, lines: &mut Vec<String>) {
    lines.push(format!("{}{}", "  ".repeat(depth), node.label));
    for child in &node.children {
        render_node(child, depth + 1, lines);
    }
}

#[cfg(test)]
mod tests {
    use super::{render_tree, TreeNode};

    #[test]
    fn tree_renders_nested_nodes() {
        let output = render_tree(&[TreeNode {
            label: "src".into(),
            children: vec![TreeNode::leaf("main.rs")],
        }]);
        assert!(output.contains("src"));
        assert!(output.contains("  main.rs"));
    }
}
