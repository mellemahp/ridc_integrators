/// Newton Raphson Solver
///
/// A multi-dimensional generalized newton-raphson root finder
///
/// This method uses an initial guess for the
///
///
// === Begin Imports ===
// std library imports
use std::f64::EPSILON;

// third party imports
extern crate nalgebra as na;
use na::allocator::Allocator;
use na::{DefaultAllocator, Dim, DimMin, DimName, DimSub, MatrixN, VectorN, U1};

// local imports
use super::finite_diff::{fdiff_jacobian, fdiff_jacobian_2};
use super::linsearch::linsrch_w_backtracking;

// === End Imports ===

// Newton raphson method using Broydens method
// see: https://en.wikipedia.org/wiki/Broyden%27s_method
//
pub fn newton_raphson_broyden<F, N: Dim + DimName + DimMin<N> + DimSub<U1>>(
    fxn: F,
    x_0: VectorN<f64, N>,
    acc: f64,
) -> Result<VectorN<f64, N>, &'static str>
where
    F: Fn(&VectorN<f64, N>) -> VectorN<f64, N>,
    DefaultAllocator: Allocator<f64, N>
        + Allocator<f64, U1, N>
        + Allocator<f64, N, N>
        + Allocator<f64, <N as DimMin<N>>::Output, N>
        + Allocator<f64, <N as DimMin<N>>::Output>
        + Allocator<f64, N, <N as DimMin<N>>::Output>
        + Allocator<f64, <<N as DimMin<N>>::Output as DimSub<U1>>::Output>,
    <N as DimMin<N>>::Output: DimName,
    <N as DimMin<N>>::Output: DimSub<U1>,
{
    const MAX_ITER: i32 = 200;
    const INV_TOL: f64 = EPSILON;
    const TOLX: f64 = 1.0_e-7_f64;

    // pre-initialize variables
    let dim = x_0.len();
    let mut f_n = fxn(&x_0);
    let mut x_last = x_0.clone();

    // check if first guess is root
    let mut test = 0.0;
    for idx in 0..dim {
        if f_n[idx].abs() > test {
            test = f_n[idx].abs()
        }
    }
    if test < 0.01 * acc {
        return Ok(x_last);
    }

    // if initial guess is not a root initialize values
    let mut jac: MatrixN<f64, N> = fdiff_jacobian_2(&fxn, &f_n, &x_0);

    // empty allocations
    let mut x_new: VectorN<f64, N>;
    let mut f_last: VectorN<f64, N>;
    let mut del_x: VectorN<f64, N>;
    let mut del_x_norm: f64;
    let mut del_f: VectorN<f64, N>;
    let mut test_f: f64;
    let mut test_x: f64;

    // Iterate to victory!
    for _ in 0..MAX_ITER {
        // update x guess
        x_new = &x_last - jac.clone().pseudo_inverse(INV_TOL)? * &f_n;

        del_x = &x_new - &x_last;
        del_x_norm = del_x.norm();

        // check for convergence of x
        test_x = 0.0;
        for idx in 0..dim {
            let temp = (del_x[idx]).abs() / (x_new[idx]).abs().max(1.0);
            if temp > test_x {
                test_x = temp;
            }
        }
        if test_x < TOLX {
            return Ok(x_last);
        }
        x_last = x_new.clone();

        // Function updates
        f_last = f_n.clone();
        f_n = fxn(&x_new);
        del_f = &f_n - &f_last;

        // check for convergence of function
        test_f = 0.0;
        for idx in 0..dim {
            if (f_n[idx]).abs() > test_f {
                test_f = f_n[idx].abs();
            }
        }
        if test_f < acc {
            return Ok(x_new);
        }
        jac = &jac + (&del_f - &jac * &del_x) / del_x_norm.powf(2.0) * &del_x.transpose();
    }
    return Err("[NEWTON BROYDEN] Maximum Number of Iterations Reached");
}

// Basic newton-raphson method using finite differencing
pub fn newton_raphson_fdiff<F, N: Dim + DimName + DimMin<N> + DimSub<U1>>(
    fxn: F,
    x_0: VectorN<f64, N>,
    acc: f64,
) -> Result<VectorN<f64, N>, &'static str>
where
    F: Fn(&VectorN<f64, N>) -> VectorN<f64, N>,
    DefaultAllocator: Allocator<f64, N>
        + Allocator<f64, N, N>
        + Allocator<f64, <N as DimMin<N>>::Output, N>
        + Allocator<f64, <N as DimMin<N>>::Output>
        + Allocator<f64, N, <N as DimMin<N>>::Output>
        + Allocator<f64, <<N as DimMin<N>>::Output as DimSub<U1>>::Output>,
    <N as DimMin<N>>::Output: DimName,
    <N as DimMin<N>>::Output: DimSub<U1>,
{
    const MAX_ITER: i32 = 200;
    const INV_TOL: f64 = EPSILON;
    const TOLX: f64 = 1.0_e-7_f64;

    // pre-initialize variables
    let mut fk = fxn(&x_0);
    let dim = x_0.len();

    // check if first guess is root
    let mut test = 0.0;
    for idx in 0..dim {
        if fk[idx].abs() > test {
            test = fk[idx].abs()
        }
    }
    if test < 0.01 * acc {
        return Ok(x_0);
    }

    // if not a root initialize other vals
    let mut jac_inv: MatrixN<f64, N> = fdiff_jacobian(&fxn, &fk, &x_0).pseudo_inverse(INV_TOL)?;
    let mut x_new: VectorN<f64, N>;
    let mut del_x: VectorN<f64, N>;
    let mut x_last = x_0.clone();
    let mut test_x: f64;
    let mut test_f: f64;

    // Iterate to victory!
    for _j in 0..MAX_ITER {
        // update x
        x_new = &x_last - jac_inv * &fk;
        del_x = &x_new - &x_last;

        // check for convergence of x
        test_x = 0.0;
        for idx in 0..dim {
            let temp = (del_x[idx]).abs() / x_new[idx].abs().max(1.0);
            if temp > test_x {
                test_x = temp;
            }
        }
        if test_x < TOLX {
            return Ok(x_last);
        }
        x_last = x_new.clone();

        // update function
        fk = fxn(&x_new);

        // check for convergence of function
        test_f = 0.0;
        for idx in 0..dim {
            if fk[idx] > test_f {
                test_f = fk[idx].abs();
            }
        }
        if test_f < acc {
            return Ok(x_new);
        }

        jac_inv = fdiff_jacobian(&fxn, &fk, &x_new).pseudo_inverse(INV_TOL)?;
    }
    return Err("Maximum Number of Iterations Reached");
}

// Basic newton-raphson method using finite differencing and a linear search method
// based off of glabally convergent method on pg 481 of Numerical Recipes
pub fn newton_raphson_linsrch<F, N: Dim + DimName + DimMin<N> + DimSub<U1>>(
    fxn: F,
    x_0: VectorN<f64, N>,
    acc: f64,
) -> Result<VectorN<f64, N>, &'static str>
where
    F: Fn(&VectorN<f64, N>) -> VectorN<f64, N>,
    DefaultAllocator: Allocator<f64, N>
        + Allocator<f64, N, N>
        + Allocator<f64, <N as DimMin<N>>::Output, N>
        + Allocator<f64, <N as DimMin<N>>::Output>
        + Allocator<f64, N, <N as DimMin<N>>::Output>
        + Allocator<f64, <<N as DimMin<N>>::Output as DimSub<U1>>::Output>,
    <N as DimMin<N>>::Output: DimName,
    <N as DimMin<N>>::Output: DimSub<U1>,
{
    // Constants
    const MAX_ITER: i32 = 200;
    const INV_TOL: f64 = EPSILON;
    const TOLX: f64 = EPSILON;
    const STEP_MAX: f64 = 100.0;

    let fmin = |x: &VectorN<f64, N>| {
        let big_f = fxn(x);
        (big_f.clone(), 0.5 * big_f.dot(&big_f))
    };

    // pre-initialize variables
    let (mut f_vec, mut f_new) = fmin(&x_0);
    let dim = x_0.len();

    // check if first guess is root
    let mut test = 0.0;
    for idx in 0..dim {
        if f_vec[idx].abs() > test {
            test = f_vec[idx].abs();
        }
    }
    if test < 0.01 * acc {
        return Ok(x_0);
    }

    // compute maximum step size for line search
    let stepmax = STEP_MAX * x_0.norm().max(dim as f64);

    // initialize other vals
    let mut jac: MatrixN<f64, N>;
    let mut x_new = x_0.clone();
    let mut x_old: VectorN<f64, N>;
    let mut p: VectorN<f64, N>;
    let mut test_x: f64;
    let mut test_f: f64;
    let mut grad: VectorN<f64, N> = VectorN::<f64, N>::repeat(0.0);
    let mut f_old: f64;
    let mut g_sum: f64;

    // Iterate to victory!
    for _j in 0..MAX_ITER {
        // calculate jacobian
        jac = fdiff_jacobian(&fxn, &f_vec, &x_new);

        // calculate gradient
        grad = &jac * &f_vec;

        // solve for p (newton step) using J * p = -F using pseudoinverse
        p = -(jac.pseudo_inverse(INV_TOL)? * &f_vec);

        // store x and f
        x_old = x_new.clone();
        f_old = f_new.clone();

        // linsearch
        let (x_out, f_vec_out, f_new_out) =
            linsrch_w_backtracking(&x_old, f_old, &grad, &mut p, stepmax, &fmin)?;

        x_new = x_out;
        f_vec = f_vec_out;
        f_new = f_new_out;

        // check for convergence of function
        test_f = 0.0;
        for idx in 0..dim {
            if f_vec[idx].abs() > test_f {
                test_f = f_vec[idx].abs();
            }
        }
        if test_f < acc {
            return Ok(x_new);
        }

        // check for convergence of x
        test_x = 0.0;
        for idx in 0..dim {
            let temp = (&x_new[idx] - &x_old[idx]).abs() / x_new[idx].abs().max(1.0);
            if temp > test_x {
                test_x = temp;
            }
        }
        if test_x < TOLX {
            return Ok(x_new);
        }
    }
    return Err("Maximum Number of Iterations Reached");
}

#[cfg(test)]
mod tests {
    use super::*;
    use na::{Matrix2, Vector1, Vector2};

    #[test]
    fn test_newton_1d() {
        let i_guess = Vector1::new(1.0);
        let fxn = |x: &Vector1<f64>| Vector1::new(x[0].powf(3.0) + 3.0 * x[0] - 7.0);

        let ans =
            newton_raphson_fdiff(fxn, i_guess, 1.0e-6_f64).expect("Couldn't converge to solution");

        // truth (from wolfram)
        let sol = 1.406287579960534691140831;
        const TOL: f64 = 1.0e-6_f64;
        assert!((ans[0] - sol).abs() < TOL);
    }

    #[test]
    fn test_newton_2d() {
        let i_guess = Vector2::new(0.0, 0.0);
        let fxn = |x: &Vector2<f64>| {
            Vector2::new(
                x[0] + 0.5 * (x[0] - x[1]).powf(3.0) - 1.0,
                0.5 * (x[1] - x[0]).powf(3.0) + x[1],
            )
        };

        // value found using scipy.optimize.root
        let ans =
            newton_raphson_fdiff(fxn, i_guess, 1.0e-6_f64).expect("Couldn't converge to solution");

        let python_sol = Vector2::new(0.8411639, 0.1588361);
        const TOL: f64 = 1.0e-7_f64;
        for idx in 0..2 {
            assert!((ans[idx] - python_sol[idx]).abs() < TOL);
        }
    }

    #[test]
    fn test_broyden_1d() {
        let i_guess = Vector1::new(1.0);
        let fxn = |x: &Vector1<f64>| Vector1::new(x[0].powf(3.0) + 3.0 * x[0] - 7.0);

        let ans = newton_raphson_broyden(fxn, i_guess, 1.0e-8_f64)
            .expect("Couldn't converge to solution");

        // truth (from wolfram)
        let sol = 1.406287579960534691140831;
        const TOL: f64 = 1.0e-7_f64;
        assert!((ans[0] - sol).abs() < TOL);
    }

    #[test]
    fn test_broyden_2d() {
        let i_guess = Vector2::new(0.0, 0.0);
        let fxn = |x: &Vector2<f64>| {
            Vector2::new(
                x[0] + 0.5 * (x[0] - x[1]).powf(3.0) - 1.0,
                0.5 * (x[1] - x[0]).powf(3.0) + x[1],
            )
        };

        // value found using scipy.optimize.root
        let ans = newton_raphson_broyden(fxn, i_guess, 1.0e-6_f64)
            .expect("Couldn't converge to solution");

        let python_sol = Vector2::new(0.8411639, 0.1588361);
        const TOL: f64 = 1.0e-7_f64;
        for idx in 0..2 {
            assert!((ans[idx] - python_sol[idx]).abs() < TOL);
        }
    }

    #[test]
    fn test_newton_linsrch_1d() {
        let i_guess = Vector1::new(1.0);
        let fxn = |x: &Vector1<f64>| Vector1::new(x[0].powf(3.0) + 3.0 * x[0] - 7.0);

        let ans = newton_raphson_linsrch(fxn, i_guess, 1.0e-6_f64)
            .expect("Couldn't converge to solution");

        // truth (from wolfram)
        let sol = 1.406287579960534691140831;
        const TOL: f64 = 1.0e-6_f64;
        assert!((ans[0] - sol).abs() < TOL);
    }

    #[test]
    fn test_newton_linsrch_2d() {
        let i_guess = Vector2::new(0.0, 0.0);
        let fxn = |x: &Vector2<f64>| {
            Vector2::new(
                x[0] + 0.5 * (x[0] - x[1]).powf(3.0) - 1.0,
                0.5 * (x[1] - x[0]).powf(3.0) + x[1],
            )
        };

        // value found using scipy.optimize.root
        let ans = newton_raphson_linsrch(fxn, i_guess, 1.0e-6_f64)
            .expect("Couldn't converge to solution");

        let python_sol = Vector2::new(0.8411639, 0.1588361);
        const TOL: f64 = 1.0e-7_f64;
        for idx in 0..2 {
            assert!((ans[idx] - python_sol[idx]).abs() < TOL);
        }
    }
}
