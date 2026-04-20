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
    for (index, node) in nodes.iter().enumerate() {
        render_node(node, "", index + 1 == nodes.len(), &mut lines);
    }
    lines.join("\n")
}

fn render_node(node: &TreeNode, prefix: &str, is_last: bool, lines: &mut Vec<String>) {
    if prefix.is_empty() {
        lines.push(node.label.clone());
    } else {
        let branch = if is_last { "`- " } else { "|- " };
        lines.push(format!("{prefix}{branch}{}", node.label));
    }

    let child_prefix = if prefix.is_empty() {
        String::new()
    } else if is_last {
        format!("{prefix}   ")
    } else {
        format!("{prefix}|  ")
    };

    for (index, child) in node.children.iter().enumerate() {
        render_node(
            child,
            if prefix.is_empty() {
                "  "
            } else {
                &child_prefix
            },
            index + 1 == node.children.len(),
            lines,
        );
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
        assert!(output.contains("`- main.rs") || output.contains("|- main.rs"));
    }
}
