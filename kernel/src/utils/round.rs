/// Yields `$lhs` devided by `$hs`, rounded up.
/// For `$lhs >= 0`, `$rhs >= 1` only.
#[macro_export]
macro_rules! div_round_up {
    ($lhs:expr, $rhs:expr) => {
        ($lhs + $rhs - 1) / $rhs
    };
}
