use std::cell::RefCell;
use std::clone::Clone;
use std::sync::Arc;

/// define `ListNode`
#[derive(Debug)]
pub(crate) struct ListNode {
    /// message id
    message_id: i32,
    /// right node pointer
    next: Option<Arc<Cell>>,
    /// left node pointer
    prev: Option<Arc<Cell>>,
}

/// define `Cell`
#[derive(Debug)]
pub(crate) struct Cell(RefCell<ListNode>);
unsafe impl Sync for Cell {}
unsafe impl Send for Cell {}
/// define `List`
#[derive(Debug)]
pub(crate) struct List {
    /// record the size
    count: i32,
    /// left head
    first: Option<Arc<Cell>>,
    /// right head
    last: Option<Arc<Cell>>,
}

impl ListNode {
    /// `new` create a Node
    pub(crate) fn create_node(message_id: i32) -> Arc<Cell> {
        Arc::new(Cell(RefCell::new(ListNode {
            message_id,
            next: None,
            prev: None,
        })))
    }
}

impl List {
    /// create a List
    pub(crate) fn new() -> List {
        let first = ListNode::create_node(0);
        let last = ListNode::create_node(0);
        first.0.borrow_mut().next = Some(Arc::clone(&last));
        last.0.borrow_mut().prev = Some(Arc::clone(&first));
        List {
            count: 0,
            first: Some(first),
            last: Some(last),
        }
    }
    /// get size of list
    pub(crate) fn list_count(&self) -> i32 {
        self.count
    }
    /// get first node
    pub(crate) fn list_first(&self) -> i32 {
        if let Some(ref f) = self.first {
            if let Some(ref rigth) = f.0.borrow().next {
                return rigth.0.borrow().message_id;
            }
        }
        -1
    }
    /// push node in first place
    pub(crate) fn list_push_first(&mut self, node: &Arc<Cell>) {
        if let Some(ref f) = self.first {
            let mut n = node.0.borrow_mut();
            n.prev = Some(Arc::clone(f));
            if let Some(ref p) = f.0.borrow().next {
                n.next = Some(Arc::clone(p));
                p.0.borrow_mut().prev = Some(Arc::clone(node));
            };
            f.0.borrow_mut().next = Some(Arc::clone(node));
        };
        self.count = self.count.wrapping_add(1);
    }
    /// pust node at last
    pub(crate) fn list_push_back(&mut self, node: &Arc<Cell>) {
        if let Some(ref l) = self.last {
            let mut n = node.0.borrow_mut();
            n.next = Some(Arc::clone(l));
            if let Some(ref p) = l.0.borrow().prev {
                n.prev = Some(Arc::clone(p));
                p.0.borrow_mut().next = Some(Arc::clone(node));
            };
            l.0.borrow_mut().prev = Some(Arc::clone(node));
        };
        self.count = self.count.wrapping_add(1);
    }
    /// pop first node
    #[allow(unused)]
    pub(crate) fn list_pop_first(&mut self) -> i32 {
        assert!((self.count != 0), "No Items for pop!");
        let mut value = 0;
        let mut pointer_pnext = None;
        if let Some(ref f) = self.first {
            if let Some(ref p) = f.0.borrow().next {
                if let Some(ref pnext) = p.0.borrow().next {
                    pointer_pnext = Some(Arc::clone(pnext));
                    pnext.0.borrow_mut().prev = Some(Arc::clone(f));
                }
                value = p.0.borrow().message_id;
            };
            f.0.borrow_mut().next = pointer_pnext;
        };
        self.count = self.count.wrapping_sub(1);
        value
    }
    /// pop last node
    #[allow(unused)]
    pub(crate) fn list_pop_last(&mut self) -> i32 {
        assert!((self.count != 0), "No Items for pop!");
        let mut value = 0;
        let mut pointer_pnext = None;
        if let Some(ref l) = self.last {
            if let Some(ref p) = l.0.borrow().prev {
                if let Some(ref pnext) = p.0.borrow().prev {
                    pointer_pnext = Some(Arc::clone(pnext));
                    pnext.0.borrow_mut().next = Some(Arc::clone(l));
                }
                value = p.0.borrow().message_id;
            };
            l.0.borrow_mut().prev = pointer_pnext;
        };
        self.count = self.count.wrapping_sub(1);
        value
    }
    /// remove a node
    pub(crate) fn remove(&mut self, node: &Arc<Cell>) {
        if let Some(ref left) = node.0.borrow().prev {
            if let Some(ref right) = node.0.borrow().next {
                left.0.borrow_mut().next = Some(Arc::clone(right));
                right.0.borrow_mut().prev = Some(Arc::clone(left));
                self.count = self.count.wrapping_sub(1);
            }
        }
    }
}

#[cfg(test)]
mod test_linked_test {
    use super::{List, ListNode};
    #[test]
    fn test_linked_list() {
        let node0 = ListNode::create_node(0);
        let node1 = ListNode::create_node(1);
        let node2 = ListNode::create_node(2);
        let mut list = List::new();
        // 1 <-> 0 <-> 2
        list.list_push_first(&node0.clone());
        list.list_push_first(&node1.clone());
        list.list_push_back(&node2.clone());
        let res0 = list.list_pop_first();
        assert_eq!(res0, 1);
        assert_eq!(list.list_count(), 2);
        let _ = list.remove(&node0.clone());
        assert_eq!(list.list_count(), 1);
        let res2 = list.list_pop_first();
        assert_eq!(res2, 2);
        assert_eq!(list.list_count(), 0);
        // 0 <-> 1
        list.list_push_back(&node0.clone());
        list.list_push_back(&node1.clone());
        let res3 = list.list_pop_last();
        assert_eq!(res3, 1);
        assert_eq!(list.list_count(), 1);
        list.remove(&node1.clone());
        assert_eq!(list.list_count(), 0);
    }
}
