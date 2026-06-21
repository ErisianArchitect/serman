

#[must_use]
#[inline(always)]
pub const fn select_copy<T: Copy>(false_: T, true_: T, condition: bool) -> T {
    [false_, true_][condition as usize]
}
