use std::mem;

pub struct IndexableSortedMap<K: Ord + Clone, V> {
    root: Option<Node<K, V>>,
}

enum Tree<K: Ord + Clone, V> {
    Leaf(K, V),
    Branch2(Node<K, V>, Node<K, V>),
    Branch3(Node<K, V>, Node<K, V>, Node<K, V>),
}

struct Node<K: Ord + Clone, V> {
    min_key: K,
    size: usize,
    tree: Box<Tree<K, V>>,
}

enum TreeContext {
    Branch2Left,
    Branch2Right,
    Branch3Left,
    Branch3Middle,
    Branch3Right,
}

struct NodeContext<'a, K: Ord + Clone, V> {
    context: TreeContext,
    node: &'a Node<K, V>,
}

pub struct TreeZipper<'a, K: Ord + Clone, V> {
    stack: Vec<NodeContext<'a, K, V>>,
    focus: (&'a K, &'a V),
}

enum InsertResult<K: Ord + Clone, V> {
    SameDepth(Node<K, V>),
    Overflow(Node<K, V>, Node<K, V>),
}

enum RemoveResult<K: Ord + Clone, V> {
    SameDepth(Node<K, V>),
    Underflow(Node<K, V>),
    Empty,
}

impl<K: Ord+Clone, V> IndexableSortedMap<K, V> {
    pub fn new() -> Self {
        IndexableSortedMap { root: None }
    }

    pub fn len(&self) -> usize {
        match &self.root {
            None => 0,
            Some(node) => node.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn lookup(&self, key: &K) -> Option<&V> {
        let mut zipper = self.zipper()?;
        zipper = zipper.advance_to(key)?;
        let (k, v) = zipper.into_focus();

        if key == k {
            Some(v)
        } else {
            None
        }
    }

    pub fn index(&self, i: usize) -> Option<(&K, &V)> {
        let mut zipper = self.zipper()?;
        zipper = zipper.advance(i)?;
        Some(zipper.into_focus())
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut map = None;
        mem::swap(&mut map, &mut self.root);

        match map {
            None => {
                self.root = Some(Node::singleton(key, value));
            },
            Some(node) => {
                match node.insert(key, value) {
                    InsertResult::SameDepth(new_node) => {
                        self.root = Some(new_node);
                    },
                    InsertResult::Overflow(left, right) => {
                        self.root = Some(Node::branch2(left, right));
                    }
                }
            },
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut map = None;
        mem::swap(&mut map, &mut self.root);

        match map {
            None => { None },
            Some(node) => {
                match node.remove(key) {
                    (RemoveResult::SameDepth(new_node), result) => {
                        self.root = Some(new_node);
                        result
                    },
                    (RemoveResult::Underflow(new_node), result) => {
                        self.root = Some(new_node);
                        result
                    },
                    (RemoveResult::Empty, result) => {
                        self.root = None;
                        result
                    },
                }
            },
        }
    }

    pub fn zipper<'a>(&'a self) -> Option<TreeZipper<'a, K, V>> {
        match &self.root {
            None => { None },
            Some(x) => { Some(x.zipper()) },
        }
    }
}

impl<K: Ord+Clone, V> Tree<K, V> {
    fn singleton(key: K, value: V) -> Tree<K, V> {
        Tree::Leaf(key, value)
    }

    fn min_key(&self) -> &K {
        match self {
            Tree::Leaf(k, _) => &k,
            Tree::Branch2(left, _) => &left.min_key,
            Tree::Branch3(left, _, _) => &left.min_key,
        }
    }

    fn len(&self) -> usize {
        match self {
            Tree::Leaf(_, _) => 1,
            Tree::Branch2(left, right) => left.len() + right.len(),
            Tree::Branch3(left, middle, right) => left.len() + middle.len() + right.len(),
        }
    }
}

impl<K: Ord+Clone, V> Node<K, V> {
    fn len(&self) -> usize {
        self.size
    }

    fn from_tree(tree: Tree<K, V>) -> Node<K, V> {
        Node {
            min_key: tree.min_key().clone(),
            size: tree.len(),
            tree: Box::new(tree),
        }
    }

    fn singleton(key: K, value: V) -> Node<K, V> {
        Node::from_tree(Tree::singleton(key, value))
    }

    fn branch2(left: Node<K, V>, right: Node<K, V>) -> Node<K, V> {
        Node::from_tree(
            Tree::Branch2(left, right)
        )
    }

    fn branch3(left: Node<K, V>, middle: Node<K, V>, right: Node<K, V>) -> Node<K, V> {
        Node::from_tree(
            Tree::Branch3(left, middle, right)
        )
    }

    fn branch4(a: Node<K, V>, b: Node<K, V>, c: Node<K, V>, d: Node<K, V>) -> Node<K, V> {
        Node::branch2(Node::branch2(a, b), Node::branch2(c, d))
    }

    fn branch5(a: Node<K, V>, b: Node<K, V>, c: Node<K, V>, d: Node<K, V>, e: Node<K, V>) -> Node<K, V> {
        Node::branch2(Node::branch2(a, b), Node::branch3(c, d, e))
    }

    fn branch6(a: Node<K, V>, b: Node<K, V>, c: Node<K, V>, d: Node<K, V>, e: Node<K, V>, f: Node<K, V>) -> Node<K, V> {
        Node::branch2(Node::branch3(a, b, c), Node::branch3(d, e, f))
    }

    fn branch7(a: Node<K, V>, b: Node<K, V>, c: Node<K, V>, d: Node<K, V>, e: Node<K, V>, f: Node<K, V>, g: Node<K, V>) -> Node<K, V> {
        Node::branch3(Node::branch2(a, b), Node::branch2(c, d), Node::branch3(e, f, g))
    }

    fn merge1(u: Node<K, V>, x: Node<K, V>, y: Node<K, V>) -> Node<K, V> {
        match (*x.tree, *y.tree) {
            (Tree::Branch2(a, b), Tree::Branch2(c, d)) => Node::branch5(u, a, b, c, d),
            (Tree::Branch2(a, b), Tree::Branch3(c, d, e)) => Node::branch6(u, a, b, c, d, e),
            (Tree::Branch3(a, b, c), Tree::Branch2(d, e)) => Node::branch6(u, a, b, c, d, e),
            (Tree::Branch3(a, b, c), Tree::Branch3(d, e, f)) => Node::branch7(u, a, b, c, d, e, f),
            _ => unreachable!(),
        }
    }

    fn merge2(x: Node<K, V>, u: Node<K, V>, y: Node<K, V>) -> Node<K, V> {
        match (*x.tree, *y.tree) {
            (Tree::Branch2(a, b), Tree::Branch2(c, d)) => Node::branch5(a, b, u, c, d),
            (Tree::Branch2(a, b), Tree::Branch3(c, d, e)) => Node::branch6(a, b, u, c, d, e),
            (Tree::Branch3(a, b, c), Tree::Branch2(d, e)) => Node::branch6(a, b, c, u, d, e),
            (Tree::Branch3(a, b, c), Tree::Branch3(d, e, f)) => Node::branch7(a, b, c, u, d, e, f),
            _ => unreachable!(),
        }
    }

    fn merge3(x: Node<K, V>, y: Node<K, V>, u: Node<K, V>) -> Node<K, V> {
        match (*x.tree, *y.tree) {
            (Tree::Branch2(a, b), Tree::Branch2(c, d)) => Node::branch5(a, b, c, d, u),
            (Tree::Branch2(a, b), Tree::Branch3(c, d, e)) => Node::branch6(a, b, c, d, e, u),
            (Tree::Branch3(a, b, c), Tree::Branch2(d, e)) => Node::branch6(a, b, c, d, e, u),
            (Tree::Branch3(a, b, c), Tree::Branch3(d, e, f)) => Node::branch7(a, b, c, d, e, f, u),
            _ => unreachable!(),
        }
    }

    fn zipper<'a>(&'a self) -> TreeZipper<'a, K, V> {
        let mut stack = Vec::new();
        let mut focus = self;

        loop {
            match focus.tree.as_ref() {
                Tree::Branch2(left, _right) => {
                    stack.push(NodeContext {
                        context: TreeContext::Branch2Left,
                        node: &focus,
                    });
                    focus = &left;
                },
                Tree::Branch3(left, _middle, _right) => {
                    stack.push(NodeContext {
                        context: TreeContext::Branch3Left,
                        node: &focus,
                    });
                    focus = &left;
                },
                Tree::Leaf(key, value) => {
                    return TreeZipper {
                        stack: stack,
                        focus: (&key, &value),
                    };
                }
            }
        }
    }

    pub fn remove(self, key: &K) -> (RemoveResult<K, V>, Option<V>) {
        match *self.tree {
            Tree::Leaf(lk, lv) => {
                if &lk == key {
                    (RemoveResult::Empty, Some(lv))
                } else {
                    (RemoveResult::SameDepth(Node::singleton(lk, lv)), None)
                }
            },
            Tree::Branch2(left, right) => {
                if key < &right.min_key {
                    match left.remove(key) {
                        (RemoveResult::Empty, result) => {
                            (RemoveResult::Underflow(right), result)
                        },
                        (RemoveResult::SameDepth(new_left), result) => {
                            (RemoveResult::SameDepth(Node::branch2(new_left, right)), result)
                        },
                        (RemoveResult::Underflow(new_left), result) => {
                            match *right.tree {
                                Tree::Leaf(_k, _v) => {
                                    unreachable!()
                                },
                                Tree::Branch2(right_left, right_right) => {
                                    (
                                        RemoveResult::Underflow(
                                            Node::branch3(new_left, right_left, right_right)
                                        ),
                                        result
                                    )
                                }
                                Tree::Branch3(right_left, right_middle, right_right) => {
                                    (
                                        RemoveResult::SameDepth(
                                            Node::branch4(
                                                new_left,
                                                right_left,
                                                right_middle,
                                                right_right,
                                            )
                                        ),
                                        result
                                    )
                                },
                            }
                        },
                    }
                } else {
                    match right.remove(key) {
                        (RemoveResult::Empty, result) => {
                            (RemoveResult::Underflow(left), result)
                        },
                        (RemoveResult::SameDepth(new_right), result) => {
                            (RemoveResult::SameDepth(Node::branch2(left, new_right)), result)
                        },
                        (RemoveResult::Underflow(new_right), result) => {
                            match *left.tree {
                                Tree::Leaf(_k, _v) => {
                                    unreachable!()
                                },
                                Tree::Branch2(left_left, left_right) => {
                                    (
                                        RemoveResult::Underflow(
                                            Node::branch3(left_left, left_right, new_right)
                                        ),
                                        result
                                    )
                                }
                                Tree::Branch3(left_left, left_middle, left_right) => {
                                    (
                                        RemoveResult::SameDepth(
                                            Node::branch4(
                                                left_left,
                                                left_middle,
                                                left_right,
                                                new_right,
                                            )
                                        ),
                                        result
                                    )
                                },
                            }
                        },
                    }
                }
            },
            Tree::Branch3(left, middle, right) => {
                if key < &right.min_key {
                    match left.remove(key) {
                        (RemoveResult::Empty, result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::branch2(middle, right)
                                ),
                                result
                            )
                        },
                        (RemoveResult::SameDepth(new_left), result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::branch3(new_left, middle, right)
                                ),
                                result
                            )
                        },
                        (RemoveResult::Underflow(new_left), result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::merge1(new_left, middle, right)
                                ),
                                result
                            )
                        }
                    }
                } else if key < &middle.min_key {
                    match middle.remove(key) {
                        (RemoveResult::Empty, result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::branch2(left, right)
                                ),
                                result
                            )
                        },
                        (RemoveResult::SameDepth(new_middle), result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::branch3(left, new_middle, right)
                                ),
                                result
                            )
                        },
                        (RemoveResult::Underflow(new_middle), result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::merge2(left, new_middle, right)
                                ),
                                result
                            )
                        }
                    }
                } else {
                    match right.remove(key) {
                        (RemoveResult::Empty, result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::branch2(left, middle)
                                ),
                                result
                            )
                        },
                        (RemoveResult::SameDepth(new_right), result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::branch3(left, middle, new_right)
                                ),
                                result
                            )
                        },
                        (RemoveResult::Underflow(new_right), result) => {
                            (
                                RemoveResult::SameDepth(
                                    Node::merge3(left, middle, new_right)
                                ),
                                result
                            )
                        }
                    }
                }
            },
        }
    }

    fn insert(self, key: K, value: V) -> InsertResult<K, V> {
        match *self.tree {
            Tree::Leaf(lk, lv) => {
                let mut left = (key, value);
                let mut right = (lk, lv);

                if right.0 < left.0 {
                    mem::swap(&mut left, &mut right);
                }

                InsertResult::Overflow(
                    Node::singleton(left.0, left.1),
                    Node::singleton(right.0, right.1),
                )
            },
            Tree::Branch2(left, right) => {
                if key < right.min_key {
                    match left.insert(key, value) {
                        InsertResult::SameDepth(new_left) => {
                            InsertResult::SameDepth(
                                Node::branch2(new_left, right)
                            )
                        },
                        InsertResult::Overflow(new_left, middle) => {
                            InsertResult::SameDepth(
                                Node::branch3(new_left, middle, right)
                            )
                        },
                    }
                } else {
                    match right.insert(key, value) {
                        InsertResult::SameDepth(new_right) => {
                            InsertResult::SameDepth(
                                Node::branch2(left, new_right)
                            )
                        },
                        InsertResult::Overflow(middle, new_right) => {
                            InsertResult::SameDepth(
                                Node::branch3(left, middle, new_right)
                            )
                        },
                    }
                }
            },
            Tree::Branch3(left, middle, right) => {
                if key < middle.min_key {
                    match left.insert(key, value) {
                        InsertResult::SameDepth(new_left) => {
                            InsertResult::SameDepth(
                                Node::branch3(new_left, middle, right)
                            )
                        },
                        InsertResult::Overflow(new_left, new_middle) => {
                            InsertResult::Overflow(
                                Node::branch2(new_left, new_middle),
                                Node::branch2(middle, right),
                            )
                        },
                    }
                } else if key < right.min_key {
                    match middle.insert(key, value) {
                        InsertResult::SameDepth(new_middle) => {
                            InsertResult::SameDepth(
                                Node::branch3(left, new_middle, right)
                            )
                        },
                        InsertResult::Overflow(l_middle, r_middle) => {
                            InsertResult::Overflow(
                                Node::branch2(left, l_middle),
                                Node::branch2(r_middle, right),
                            )
                        },
                    }
                } else {
                    match right.insert(key, value) {
                        InsertResult::SameDepth(new_right) => {
                            InsertResult::SameDepth(
                                Node::branch3(left, middle, new_right)
                            )
                        },
                        InsertResult::Overflow(new_middle, new_right) => {
                            InsertResult::Overflow(
                                Node::branch2(left, middle),
                                Node::branch2(new_middle, new_right),
                            )
                        },
                    }
                }
            },
        }
    }
}

impl<'a, K: Ord + Clone, V> TreeZipper<'a, K, V> {
    pub fn into_focus(self) -> (&'a K, &'a V) {
        self.focus
    }

    pub fn focus(&self) -> (&K, &V) {
        (&self.focus.0, &self.focus.1)
    }

    pub fn advance_to(mut self, k: &K) -> Option<TreeZipper<'a, K, V>> {
        if self.focus.0 >= k {
            return Some(self);
        }

        let mut focus = loop {
            match self.stack.pop() {
                None => { return None; },
                Some(context) => {
                    match context.context {
                        TreeContext::Branch2Left => {
                            if let Tree::Branch2(_left, right) = context.node.tree.as_ref() {
                                self.stack.push(NodeContext {
                                    context: TreeContext::Branch2Right,
                                    node: context.node,
                                });

                                if &right.min_key >= k {
                                    break right;
                                }
                            } else {
                                unreachable!()
                            }
                        },
                        TreeContext::Branch2Right => {},
                        TreeContext::Branch3Left => {
                            if let Tree::Branch3(_left, middle, _right) = context.node.tree.as_ref() {
                                self.stack.push(NodeContext {
                                    context: TreeContext::Branch3Middle,
                                    node: context.node,
                                });

                                if &middle.min_key >= k {
                                    break middle;
                                }
                            } else {
                                unreachable!()
                            }
                        },
                        TreeContext::Branch3Middle => {
                            if let Tree::Branch3(_left, _middle, right) = context.node.tree.as_ref() {
                                self.stack.push(NodeContext {
                                    context: TreeContext::Branch3Right,
                                    node: context.node,
                                });

                                if &right.min_key >= k {
                                    break right;
                                }
                            } else {
                                unreachable!()
                            }
                        },
                        TreeContext::Branch3Right => {},
                    }
                }
            }
        };

        loop {
            match focus.tree.as_ref() {
                Tree::Branch2(left, right) => {
                    if &left.min_key >= k {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch2Left,
                            node: &focus,
                        });

                        focus = &left;
                    } else {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch2Right,
                            node: &focus,
                        });

                        focus = &right;
                    }
                },
                Tree::Branch3(left, middle, right) => {
                    if &left.min_key >= k {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch3Left,
                            node: &focus,
                        });

                        focus = &left;
                    } else if &middle.min_key >= k {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch3Middle,
                            node: &focus,
                        });

                        focus = &middle;
                    } else {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch3Right,
                            node: &focus,
                        });

                        focus = &right;
                    }
                },
                Tree::Leaf(key, value) => {
                    self.focus = (&key, &value);
                    break;
                }
            }
        }

        Some(self)
    }

    pub fn advance(mut self, mut n: usize) -> Option<TreeZipper<'a, K, V>> {
        if n == 0 {
            return Some(self);
        } else {
            n -= 1;
        }

        let mut focus = loop {
            match self.stack.pop() {
                None => return None,
                Some(context) => {
                    match context.context {
                        TreeContext::Branch2Left => {
                            if let Tree::Branch2(_left, right) = context.node.tree.as_ref() {
                                self.stack.push(NodeContext {
                                    context: TreeContext::Branch2Right,
                                    node: context.node,
                                });

                                if n < right.size {
                                    break right;
                                } else {
                                    n -= right.size;
                                }
                            } else {
                                unreachable!()
                            }
                        },
                        TreeContext::Branch2Right => {},
                        TreeContext::Branch3Left => {
                            if let Tree::Branch3(_left, middle, _right) = context.node.tree.as_ref() {
                                self.stack.push(NodeContext {
                                    context: TreeContext::Branch3Middle,
                                    node: context.node,
                                });

                                if n < middle.size {
                                    break middle;
                                } else {
                                    n -= middle.size;
                                }
                            } else {
                                unreachable!()
                            }
                        },
                        TreeContext::Branch3Middle => {
                            if let Tree::Branch3(_left, _middle, right) = context.node.tree.as_ref() {
                                self.stack.push(NodeContext {
                                    context: TreeContext::Branch3Right,
                                    node: context.node,
                                });

                                if n < right.size {
                                    break right;
                                } else {
                                    n -= right.size;
                                }
                            } else {
                                unreachable!()
                            }
                        },
                        TreeContext::Branch3Right => {},
                    }
                },
            }
        };

        loop {
            match focus.tree.as_ref() {
                Tree::Branch2(left, right) => {
                    if n < left.size {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch2Left,
                            node: &focus,
                        });

                        focus = &left;
                    } else {
                        n -= left.size;

                        self.stack.push(NodeContext {
                            context: TreeContext::Branch2Right,
                            node: &focus,
                        });

                        focus = &right;
                    }
                },
                Tree::Branch3(left, middle, right) => {
                    if n < left.size {
                        self.stack.push(NodeContext {
                            context: TreeContext::Branch3Left,
                            node: &focus,
                        });

                        focus = &left;
                    } else {
                        n -= left.size;

                        if n < middle.size {
                            self.stack.push(NodeContext {
                                context: TreeContext::Branch3Middle,
                                node: &focus,
                            });

                            focus = &middle;
                        } else {
                            n -= middle.size;

                            self.stack.push(NodeContext {
                                context: TreeContext::Branch3Right,
                                node: &focus,
                            });

                            focus = &right;
                        }
                    }
                },
                Tree::Leaf(key, value) => {
                    self.focus = (&key, &value);
                    break;
                }
            }
        }

        Some(self)
    }
}
