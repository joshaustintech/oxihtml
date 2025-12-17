pub type NodeId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualName {
    pub ns: Namespace,
    pub local: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attr {
    pub name: QualName,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doctype {
    pub name: String,
    pub public_id: String,
    pub system_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeData {
    Document,
    DocumentFragment,
    Element {
        name: QualName,
        attrs: Vec<Attr>,
        template_contents: Option<NodeId>,
    },
    Text(String),
    Comment(String),
    Doctype(Doctype),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub data: NodeData,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub arena: Vec<Node>,
    pub root: NodeId,
}

#[derive(Clone, Debug)]
pub struct DocumentFragment {
    pub arena: Vec<Node>,
    pub root: NodeId,
}

impl Document {
    pub fn new_empty() -> Self {
        let mut arena = Vec::new();
        let root = arena.len();
        arena.push(Node {
            data: NodeData::Document,
            parent: None,
            children: Vec::new(),
        });
        Self { arena, root }
    }
}

impl DocumentFragment {
    pub fn new_empty() -> Self {
        let mut arena = Vec::new();
        let root = arena.len();
        arena.push(Node {
            data: NodeData::DocumentFragment,
            parent: None,
            children: Vec::new(),
        });
        Self { arena, root }
    }
}

pub fn append_child(arena: &mut Vec<Node>, parent: NodeId, child: NodeId) {
    arena[child].parent = Some(parent);
    arena[parent].children.push(child);
}

