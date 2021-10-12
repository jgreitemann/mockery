use std::ops::Range;
use std::path::PathBuf;

pub trait Node: Sized + Clone + 'static {
    type Item;

    fn content(&self) -> Self::Item;
    fn parent(&self) -> Option<Self>;
    fn children(&self) -> Box<dyn Iterator<Item = Self>>;

    fn search(&self, radius: usize) -> Box<dyn Iterator<Item = Self::Item>> {
        let (depth, start) = Parents(Some(self.clone()))
            .enumerate()
            .take(radius + 1)
            .last()
            .unwrap();

        Box::new(BreadthFirstTreeIter {
            nodes: vec![start],
            index: 0,
            levels: (0..(depth + radius)),
        })
    }
}

struct Parents<N: Node>(Option<N>);

impl<N: Node> Iterator for Parents<N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let parent = self.0.as_ref().and_then(|n| n.parent());
        std::mem::replace(&mut self.0, parent)
    }
}

struct BreadthFirstTreeIter<N: Node> {
    nodes: Vec<N>,
    index: usize,
    levels: Range<usize>,
}

impl<N: Node> Iterator for BreadthFirstTreeIter<N> {
    type Item = N::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.nodes.is_empty() {
            None
        } else if self.index < self.nodes.len() {
            let next_index = self.index + 1;
            Some(self.nodes[std::mem::replace(&mut self.index, next_index)].content())
        } else if !self.levels.is_empty() {
            self.nodes = self.nodes.iter().flat_map(|n| n.children()).collect();
            self.levels = (self.levels.start + 1)..self.levels.end;
            self.index = 0;
            self.next()
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct FilesystemDirectoryNode {
    pub path: PathBuf,
}

impl Node for FilesystemDirectoryNode {
    type Item = PathBuf;

    fn content(&self) -> Self::Item {
        self.path.clone()
    }

    fn parent(&self) -> Option<Self> {
        self.path
            .parent()
            .map(|p| p.to_path_buf())
            .map(|path| FilesystemDirectoryNode { path })
    }

    fn children(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.path
                .read_dir()
                .unwrap()
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .map(|path| FilesystemDirectoryNode { path }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone)]
    struct TestTreeNode {
        content: i32,
        children: Vec<TestTreeNode>,
    }

    #[derive(Clone)]
    struct TestNode {
        root: TestTreeNode,
        indices: Vec<usize>,
    }

    impl TestNode {
        fn this_node(&self) -> &TestTreeNode {
            self.indices
                .iter()
                .fold(&self.root, |node, &idx| &node.children[idx])
        }
    }

    impl Node for TestNode {
        type Item = i32;

        fn content(&self) -> Self::Item {
            self.this_node().content
        }

        fn parent(&self) -> Option<Self> {
            if self.indices.is_empty() {
                None
            } else {
                Some(TestNode {
                    root: self.root.clone(),
                    indices: self.indices[0..self.indices.len() - 1].to_vec(),
                })
            }
        }

        fn children(&self) -> Box<dyn Iterator<Item = Self>> {
            let n = self.this_node();
            let orig_indices = self.indices.clone();
            let root = self.root.clone();
            Box::new((0..n.children.len()).map(move |i| {
                let mut indices = orig_indices.clone();
                indices.push(i);
                TestNode {
                    root: root.clone(),
                    indices,
                }
            }))
        }
    }

    fn get_test_tree() -> TestTreeNode {
        // 0
        // +--1
        // |  +--2
        // |  +--3
        // |  |  +--4
        // |  +--5
        // +--6
        // +--7
        //    +--8
        //       +--9
        TestTreeNode {
            content: 0,
            children: vec![
                TestTreeNode {
                    content: 1,
                    children: vec![
                        TestTreeNode {
                            content: 2,
                            children: vec![],
                        },
                        TestTreeNode {
                            content: 3,
                            children: vec![TestTreeNode {
                                content: 4,
                                children: vec![],
                            }],
                        },
                        TestTreeNode {
                            content: 5,
                            children: vec![],
                        },
                    ],
                },
                TestTreeNode {
                    content: 6,
                    children: vec![],
                },
                TestTreeNode {
                    content: 7,
                    children: vec![TestTreeNode {
                        content: 8,
                        children: vec![TestTreeNode {
                            content: 9,
                            children: vec![],
                        }],
                    }],
                },
            ],
        }
    }

    #[test]
    fn content_of_root() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert_eq!(root.content(), 0);
    }

    #[test]
    fn content_of_leaf() {
        let leaf = TestNode {
            root: get_test_tree(),
            indices: vec![0, 1, 0],
        };
        assert_eq!(leaf.content(), 4);
    }

    #[test]
    fn children_of_root() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert_eq!(
            root.children().map(|n| n.content()).collect::<Vec<_>>(),
            vec![1, 6, 7]
        );
    }

    #[test]
    fn parent_of_root() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert!(root.parent().is_none());
    }

    #[test]
    fn parent_of_branch() {
        let leaf = TestNode {
            root: get_test_tree(),
            indices: vec![0, 1],
        };
        assert_eq!(leaf.parent().map(|n| n.content()), Some(1));
    }

    #[test]
    fn parent_of_leaf() {
        let leaf = TestNode {
            root: get_test_tree(),
            indices: vec![0, 1, 0],
        };
        assert_eq!(leaf.parent().map(|n| n.content()), Some(3));
    }

    #[test]
    fn search_around_root_with_radius_0() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert_eq!(root.search(0).collect::<Vec<_>>(), vec![0]);
    }

    #[test]
    fn search_around_root_with_radius_1() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert_eq!(root.search(1).collect::<Vec<_>>(), vec![0, 1, 6, 7]);
    }

    #[test]
    fn search_around_root_with_radius_2() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert_eq!(
            root.search(2).collect::<Vec<_>>(),
            vec![0, 1, 6, 7, 2, 3, 5, 8]
        );
    }

    #[test]
    fn search_around_root_with_radius_3() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![],
        };
        assert_eq!(
            root.search(3).collect::<Vec<_>>(),
            vec![0, 1, 6, 7, 2, 3, 5, 8, 4, 9]
        );
    }

    #[test]
    fn search_around_branch_with_radius_0() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![2],
        };
        assert_eq!(root.search(0).collect::<Vec<_>>(), vec![7]);
    }

    #[test]
    fn search_around_branch_with_radius_1() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![2],
        };
        assert_eq!(
            root.search(1).collect::<Vec<_>>(),
            vec![0, 1, 6, 7, 2, 3, 5, 8]
        );
    }

    #[test]
    fn search_around_branch_with_radius_2() {
        let root = TestNode {
            root: get_test_tree(),
            indices: vec![2],
        };
        assert_eq!(
            root.search(2).collect::<Vec<_>>(),
            vec![0, 1, 6, 7, 2, 3, 5, 8, 4, 9]
        );
    }
}
