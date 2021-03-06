use std::num::{Zero, Float};
use traits::operations::{Transpose, ApproxEq};
use traits::structure::{ColSlice, Eye, Indexable, Diag};
use traits::geometry::Norm;
use std::cmp::min;

/// Get the householder matrix corresponding to a reflexion to the hyperplane
/// defined by `vec`. It can be a reflexion contained in a subspace.
///
/// # Arguments
/// * `dim` - the dimension of the space the resulting matrix operates in
/// * `start` - the starting dimension of the subspace of the reflexion
/// * `vec` - the vector defining the reflection.
pub fn householder_matrix<N: Float,
                          M: Eye + Indexable<(uint, uint), N>,
                          V: Indexable<uint, N>>
                          (dim: uint, start: uint, vec: V) -> M {
    let mut qk : M = Eye::new_identity(dim);
    let subdim = vec.shape();

    let stop = subdim + start;

    assert!(dim >= stop);

    for j in range(start, stop) {
        for i in range(start, stop) {
            unsafe {
                let vv = vec.unsafe_at(i - start) * vec.unsafe_at(j - start);
                let qkij = qk.unsafe_at((i, j));
                qk.unsafe_set((i, j), qkij - vv - vv);
            }
        }
    }
    qk
}

/// QR decomposition using Householder reflections.
///
/// # Arguments
/// * `m` - matrix to decompose
pub fn qr<N: Float,
          V: Indexable<uint, N> + Norm<N>,
          M: Clone + Eye + ColSlice<V> + Transpose
              + Indexable<(uint, uint), N> + Mul<M, M>>
          (m: &M) 
          -> (M, M) {
    let (rows, cols) = m.shape();
    assert!(rows >= cols);
    let mut q : M = Eye::new_identity(rows);
    let mut r = m.clone();

    let iterations = min(rows - 1, cols);

    for ite in range(0u, iterations) {
        let mut v = r.col_slice(ite, ite, rows);
        let alpha =
            if unsafe { v.unsafe_at(ite) } >= Zero::zero() {
                -Norm::norm(&v)
            }
            else {
                Norm::norm(&v)
            };
        unsafe {
            let x = v.unsafe_at(0);
            v.unsafe_set(0, x - alpha);
        }
        if !v.normalize().is_zero() {
            let qk: M = householder_matrix(rows, ite, v);
            r = qk * r;
            q = q * Transpose::transpose_cpy(&qk);
        }
    }

    (q, r)
}

/// Eigendecomposition of a square matrix using the qr algorithm.
pub fn eigen_qr<N:  Float,
                V:  Indexable<uint, N> + Norm<N>,
                V2: Zero,
                M:  Clone + Eye + ColSlice<V> + Transpose
                    + Indexable<(uint, uint), N> + Mul<M, M>
                    + Diag<V2> + ApproxEq<N> + Add<M, M>
                    + Sub<M, M>>
                (m: &M, eps: &N, niter: uint) -> (M, V2) {
    let (rows, cols) = m.shape();

    assert!(rows == cols, "The matrix being decomposed must be square.");

    let mut eigenvectors: M = Eye::new_identity(rows);
    let mut eigenvalues = m.clone();
    let mut shifter: M = Eye::new_identity(rows);

    let mut iter = 0u;
    for _ in range(0, niter) {
        let mut stop = true;

        for j in range(0, cols) {
            for i in range(0, j) {
                if unsafe { eigenvalues.unsafe_at((i, j)) }.abs() >= *eps {
                    stop = false;
                    break;
                }
            }

            for i in range(j + 1, rows) {
                if unsafe { eigenvalues.unsafe_at((i, j)) }.abs() >= *eps {
                    stop = false;
                    break;
                }
            }
        }

        if stop {
            break;
        }
        iter = iter + 1;

        // FIXME: This is a very naive implementation.
        let shift = unsafe { eigenvalues.unsafe_at((rows - 1, rows - 1)) };

        for i in range(0, rows) {
            unsafe { shifter.unsafe_set((i, i), shift.clone()) }
        }

        let (q, r) = qr(&eigenvalues);//  - shifter));

        eigenvalues = r * q /*+ shifter*/;
        eigenvectors = eigenvectors * q;
    }

    (eigenvectors, eigenvalues.diag())
}
