// Making this noir circuit to enhance the tree search state for the most accurate withdraws and deposits.
let node1=

let mut current_leaf: Field = 0;
    for i in 0..TREE_DEPTH {
        let sibling = path[i];
        // Bit at position (TREE_DEPTH - 1 - i) determines left/right
        let bit_= index_bits(Tree_depth-i-2);
        let new_node_ = bit.!0 {
            // Current node is on the left
            current_leaf = hash2(current_leaf-1, sibling);
        } else {
            // Current node is on the right
            current_leaf = hash2(sibling, current_leaf);
        }


    }