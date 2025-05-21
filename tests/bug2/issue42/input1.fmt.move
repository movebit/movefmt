module red_black_map::red_black_map {

    use std::vector;

    enum Color has copy, drop {
        Red,
        Black
    }

    const NIL: u64 = 0xffffffffffffffff;

    const LEFT: u64 = 0;
    const MINIMUM: u64 = 0;
    const PREDECESSOR: u64 = 0;

    const RIGHT: u64 = 1;
    const MAXIMUM: u64 = 1;
    const SUCCESSOR: u64 = 1;

    /// Map key already exists.
    const E_KEY_ALREADY_EXISTS: u64 = 0;
    /// Map key not found.
    const E_KEY_NOT_FOUND: u64 = 1;
    /// Map is empty.
    const E_EMPTY: u64 = 2;
    /// No predecessor or successor.
    const E_UNABLE_TO_TRAVERSE: u64 = 3;

    struct Node<V> {
        key: u256,
        value: V,
        color: Color,
        parent: u64,
        children: vector<u64>
    }

    struct Map<V> {
        root: u64,
        nodes: vector<Node<V>>
    }

    public fun remove<V>(self: &mut Map<V>, key: u256): V {
        let (node_index, parent_index, child_direction) = self.search(key);
        assert!(node_index != NIL, E_KEY_NOT_FOUND);

        // Borrow node, inspect fields.
        let nodes_ref_mut = &mut self.nodes;
        let node_ref_mut = &mut nodes_ref_mut[node_index];
        let left_child_index = node_ref_mut.children[LEFT];
        let right_child_index = node_ref_mut.children[RIGHT];

        // Simple case 1: node has 2 children, will fall through to another case after swap.
        if (left_child_index != NIL && right_child_index != NIL) {

            let node_color = node_ref_mut.color; // Store node's color for tree position swap.

            // Identify successor, the leftmost child of node's right subtree.
            let successor_index = right_child_index;

            // Reassign local variables for fallthrough to delete relocated node at successor tree
            // position. Note that child direction of successor position is originally right only
            // when the successor loop does not iterate past the right child of the original node,
            // e.g. when the successor is the only node in the right subtree of the original node.
            child_direction =
                if (right_child_index == successor_index) RIGHT else LEFT;
        };
        // Simple case 2: node has 1 child.
        if (left_child_index != NIL || right_child_index != NIL) {
            let child_index =
                if (left_child_index != NIL) left_child_index
                else right_child_index;

            // Replace node with its child, which is then colored black.
            let child_ref_mut = &mut nodes_ref_mut[child_index];
            child_ref_mut.parent = parent_index;
            child_ref_mut.color = Color::Black;
            nodes_ref_mut.swap(node_index, child_index);

            node_index = child_index; // Flag updated index for deallocation.
            // From now on node has no children.
        } else { // Complex case: black non-root leaf.

            // Replace node at its parent by NIL.
            let parent_ref_mut = &mut nodes_ref_mut[parent_index];
            parent_ref_mut.children[child_direction] = NIL;

            // Declare loop variables.
            let sibling_index;

            loop {
                // Case_D2.
                nodes_ref_mut[sibling_index].color = Color::Red;
                let new_node_index = parent_index;
                parent_index = nodes_ref_mut[new_node_index].parent;
                if (parent_index == NIL) break;
                parent_ref_mut = &mut nodes_ref_mut[parent_index];
                child_direction =
                    if (new_node_index == parent_ref_mut.children[LEFT]) LEFT
                    else RIGHT;
            }; // Case_D1.
        };
    }
}
