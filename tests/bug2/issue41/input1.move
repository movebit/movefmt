module test {
inline fun child_on_side_mut<V: drop>(self: &mut Node<V>, side: Side): &mut Pointer {
        if (side is Side::Left)&mut self.left else &mut self.right
    }
}