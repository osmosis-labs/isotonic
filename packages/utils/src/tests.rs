use cosmwasm_std::{Decimal, Uint128};

/// Performs "almost exact" comparison between numbers.
///
/// The exact check is: `abs((l - r) / l) < ratio)`, so it is relative difference check.
/// As it is meant to be used for tokens calculations, it is using `Decimal` as ratio (or something
/// implementing `Into<Decimal>`)
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $ratio:expr $(,)?) => {
        if let Err((diff, rel)) = $crate::tests::check_approx_eq_impl($left, $right, $ratio) {
            panic!(
                "Assertion {0} ~= {1} failed: values are not close enough\n{0} = {2}\n{1} = {3}\n|{2}-{3}| = {4}\n{4}/{2} = {5}\n{5} > {6}",
                stringify!($left), stringify!($right),
                $left, $right,
                diff, rel, $ratio
            );
        }
    };
    ($left:expr, $right:expr, $ratio:expr, $($arg:tt)+) => {
        if let Err((diff, rel)) = $crate::tests::check_approx_eq_impl($left, $right, $ratio) {
            panic!(format!($($tt)*));
        }
    }
}

pub fn check_approx_eq_impl(
    left: impl Into<Uint128>,
    right: impl Into<Uint128>,
    ratio: impl Into<Decimal>,
) -> Result<(), (Uint128, Decimal)> {
    let left = left.into();
    let right = right.into();
    let ratio = ratio.into();

    let (l, r) = if left < right {
        (left, right)
    } else {
        (right, left)
    };

    let diff = r - l;
    let rel = Decimal::from_ratio(diff, left);

    if rel < ratio {
        Ok(())
    } else {
        Err((diff, rel))
    }
}
