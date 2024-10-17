use anyhow::bail;
use usvg::{Group, Node, Path};

pub fn find_path(g: &Group) -> anyhow::Result<&Path> {
    for node in g.children() {
        match node {
            Node::Group(ref group) => {
                return find_path(group);
            },
            Node::Path(ref path) => {
                if path.stroke().is_some() {
                    return Ok(path);
                }
            },
            Node::Image(_) => {
                bail!("Images will not be supported, please use a single vector path svg :(");
            },
            Node::Text(_) => {
                bail!("Text will not be supported, please use a single vector path svg :(");
            }
        }
    }
    bail!("No valid path could be found")
}