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
        let nodes_ref_mut = &mut self.nodes;
        let node_ref_mut = &mut nodes_ref_mut[node_index];
        let left_child_index = node_ref_mut.children[LEFT];
        let right_child_index = node_ref_mut.children[RIGHT];
        if (left_child_index != NIL) {
            let successor_index = right_child_index;
            // xxx
            child_direction =
                if (right_child_index == successor_index) RIGHT else LEFT;
        };
    }
}
