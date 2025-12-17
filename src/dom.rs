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

    pub fn create_element(&mut self, name: QualName) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Element {
                name,
                attrs: Vec::new(),
                template_contents: None,
            },
            parent: None,
            children: Vec::new(),
        });
        id
    }

    pub fn create_text(&mut self, data: impl Into<String>) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Text(data.into()),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    pub fn create_comment(&mut self, data: impl Into<String>) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Comment(data.into()),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    pub fn create_doctype(&mut self, doctype: Doctype) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Doctype(doctype),
            parent: None,
            children: Vec::new(),
        });
        id
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

    pub fn create_element(&mut self, name: QualName) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Element {
                name,
                attrs: Vec::new(),
                template_contents: None,
            },
            parent: None,
            children: Vec::new(),
        });
        id
    }

    pub fn create_text(&mut self, data: impl Into<String>) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Text(data.into()),
            parent: None,
            children: Vec::new(),
        });
        id
    }

    pub fn create_comment(&mut self, data: impl Into<String>) -> NodeId {
        let id = self.arena.len();
        self.arena.push(Node {
            data: NodeData::Comment(data.into()),
            parent: None,
            children: Vec::new(),
        });
        id
    }
}

pub fn append_child(arena: &mut Vec<Node>, parent: NodeId, child: NodeId) {
    arena[child].parent = Some(parent);
    arena[parent].children.push(child);
}

pub fn insert_before(arena: &mut Vec<Node>, parent: NodeId, new_child: NodeId, reference: Option<NodeId>) {
    if let Some(r) = reference {
        let pos = arena[parent].children.iter().position(|&c| c == r);
        if let Some(i) = pos {
            arena[new_child].parent = Some(parent);
            arena[parent].children.insert(i, new_child);
            return;
        }
    }
    append_child(arena, parent, new_child);
}

pub fn detach(arena: &mut Vec<Node>, node: NodeId) {
    let Some(parent) = arena[node].parent else {
        return;
    };
    if let Some(pos) = arena[parent].children.iter().position(|&c| c == node) {
        arena[parent].children.remove(pos);
    }
    arena[node].parent = None;
}

pub fn set_attr(arena: &mut Vec<Node>, element: NodeId, attr: Attr) {
    let NodeData::Element { attrs, .. } = &mut arena[element].data else {
        return;
    };
    if let Some(existing) = attrs.iter_mut().find(|a| a.name == attr.name) {
        existing.value = attr.value;
        return;
    }
    attrs.push(attr);
}

pub fn ensure_template_contents(arena: &mut Vec<Node>, template: NodeId) -> NodeId {
    let existing = match &arena[template].data {
        NodeData::Element {
            template_contents, ..
        } => *template_contents,
        _ => return template,
    };
    if let Some(id) = existing {
        return id;
    }

    let id = arena.len();
    arena.push(Node {
        data: NodeData::DocumentFragment,
        parent: None,
        children: Vec::new(),
    });

    if let NodeData::Element {
        template_contents, ..
    } = &mut arena[template].data
    {
        *template_contents = Some(id);
    }
    id
}
