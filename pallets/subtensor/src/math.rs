// we get a compiler warning for this , even though  the trait is used in the
// quantile function.
use crate::alloc::borrow::ToOwned;
#[allow(unused)]
use num_traits::float::Float;
use sp_runtime::traits::{CheckedAdd, Saturating};
use sp_std::cmp::Ordering;

use sp_std::vec;
use substrate_fixed::transcendental::{exp, ln};
use substrate_fixed::types::{I32F32, I64F64};

// TODO: figure out what cfg gate this needs to not be a warning in rustc
#[allow(unused)]
use sp_std::vec::Vec;

#[allow(dead_code)]
pub fn fixed(val: f32) -> I32F32 {
    I32F32::from_num(val)
}

#[allow(dead_code)]
pub fn fixed_to_u16(x: I32F32) -> u16 {
    x.to_num::<u16>()
}

#[allow(dead_code)]
pub fn fixed_to_u64(x: I32F32) -> u64 {
    x.to_num::<u64>()
}

#[allow(dead_code)]
pub fn fixed64_to_u64(x: I64F64) -> u64 {
    x.to_num::<u64>()
}

#[allow(dead_code)]
pub fn fixed64_to_fixed32(x: I64F64) -> I32F32 {
    I32F32::from_num(x)
}

#[allow(dead_code)]
pub fn fixed32_to_fixed64(x: I32F32) -> I64F64 {
    I64F64::from_num(x)
}

#[allow(dead_code)]
pub fn u16_to_fixed(x: u16) -> I32F32 {
    I32F32::from_num(x)
}

#[allow(dead_code)]
pub fn u16_proportion_to_fixed(x: u16) -> I32F32 {
    I32F32::from_num(x).saturating_div(I32F32::from_num(u16::MAX))
}

#[allow(dead_code)]
pub fn fixed_proportion_to_u16(x: I32F32) -> u16 {
    fixed_to_u16(x.saturating_mul(I32F32::from_num(u16::MAX)))
}

#[allow(dead_code)]
pub fn vec_fixed32_to_u64(vec: Vec<I32F32>) -> Vec<u64> {
    vec.into_iter().map(fixed_to_u64).collect()
}

#[allow(dead_code)]
pub fn vec_fixed64_to_fixed32(vec: Vec<I64F64>) -> Vec<I32F32> {
    vec.into_iter().map(fixed64_to_fixed32).collect()
}

#[allow(dead_code)]
pub fn vec_fixed32_to_fixed64(vec: Vec<I32F32>) -> Vec<I64F64> {
    vec.into_iter().map(fixed32_to_fixed64).collect()
}

#[allow(dead_code)]
pub fn vec_fixed64_to_u64(vec: Vec<I64F64>) -> Vec<u64> {
    vec.into_iter().map(fixed64_to_u64).collect()
}

#[allow(dead_code)]
pub fn vec_u16_proportions_to_fixed(vec: Vec<u16>) -> Vec<I32F32> {
    vec.into_iter().map(u16_proportion_to_fixed).collect()
}

#[allow(dead_code)]
pub fn vec_fixed_proportions_to_u16(vec: Vec<I32F32>) -> Vec<u16> {
    vec.into_iter().map(fixed_proportion_to_u16).collect()
}

#[allow(dead_code)]
// Max-upscale vector and convert to u16 so max_value = u16::MAX. Assumes non-negative normalized input.
pub fn vec_max_upscale_to_u16(vec: &[I32F32]) -> Vec<u16> {
    let u16_max: I32F32 = I32F32::from_num(u16::MAX);
    let threshold: I32F32 = I32F32::from_num(32768);
    let max_value: Option<&I32F32> = vec.iter().max();
    match max_value {
        Some(val) => {
            if *val == I32F32::from_num(0) {
                return vec
                    .iter()
                    .map(|e: &I32F32| e.saturating_mul(u16_max).to_num::<u16>())
                    .collect();
            }
            if *val > threshold {
                return vec
                    .iter()
                    .map(|e: &I32F32| {
                        e.saturating_mul(u16_max.saturating_div(*val))
                            .round()
                            .to_num::<u16>()
                    })
                    .collect();
            }
            return vec
                .iter()
                .map(|e: &I32F32| {
                    e.saturating_mul(u16_max)
                        .saturating_div(*val)
                        .round()
                        .to_num::<u16>()
                })
                .collect();
        }
        None => {
            let sum: I32F32 = vec.iter().sum();
            return vec
                .iter()
                .map(|e: &I32F32| {
                    e.saturating_mul(u16_max)
                        .saturating_div(sum)
                        .to_num::<u16>()
                })
                .collect();
        }
    }
}

#[allow(dead_code)]
// Max-upscale u16 vector and convert to u16 so max_value = u16::MAX. Assumes u16 vector input.
pub fn vec_u16_max_upscale_to_u16(vec: &[u16]) -> Vec<u16> {
    let vec_fixed: Vec<I32F32> = vec.iter().map(|e: &u16| I32F32::from_num(*e)).collect();
    vec_max_upscale_to_u16(&vec_fixed)
}

#[allow(dead_code)]
// Checks if u16 vector, when normalized, has a max value not greater than a u16 ratio max_limit.
pub fn check_vec_max_limited(vec: &[u16], max_limit: u16) -> bool {
    let max_limit_fixed: I32F32 =
        I32F32::from_num(max_limit).saturating_div(I32F32::from_num(u16::MAX));
    let mut vec_fixed: Vec<I32F32> = vec.iter().map(|e: &u16| I32F32::from_num(*e)).collect();
    inplace_normalize(&mut vec_fixed);
    let max_value: Option<&I32F32> = vec_fixed.iter().max();
    max_value.map_or(true, |v| *v <= max_limit_fixed)
}

#[allow(dead_code)]
pub fn sum(x: &[I32F32]) -> I32F32 {
    x.iter().sum()
}

#[allow(dead_code)]
// Sums a Vector of type that has CheckedAdd trait.
// Returns None if overflow occurs during sum using T::checked_add.
// Returns Some(T::default()) if input vector is empty.
pub fn checked_sum<T>(x: &[T]) -> Option<T>
where
    T: Copy + Default + CheckedAdd,
{
    let mut iter = x.iter();
    let Some(mut sum) = iter.next().copied() else {
        return Some(T::default());
    };
    for i in iter {
        sum = sum.checked_add(i)?;
    }
    Some(sum)
}

// Return true when vector sum is zero.
#[allow(dead_code)]
pub fn is_zero(vector: &[I32F32]) -> bool {
    let vector_sum: I32F32 = sum(vector);
    vector_sum == I32F32::from_num(0)
}

// Exp safe function with I32F32 output of I32F32 input.
#[allow(dead_code)]
pub fn exp_safe(input: I32F32) -> I32F32 {
    let min_input: I32F32 = I32F32::from_num(-20); // <= 1/exp(-20) = 485 165 195,4097903
    let max_input: I32F32 = I32F32::from_num(20); // <= exp(20) = 485 165 195,4097903
    let mut safe_input: I32F32 = input;
    if input < min_input {
        safe_input = min_input;
    } else if max_input < input {
        safe_input = max_input;
    }
    let output: I32F32;
    match exp(safe_input) {
        Ok(val) => {
            output = val;
        }
        Err(_err) => {
            if safe_input <= 0 {
                output = I32F32::from_num(0);
            } else {
                output = I32F32::max_value();
            }
        }
    }
    output
}

// Sigmoid safe function with I32F32 output of I32F32 input with offset kappa and (recommended) scaling 0 < rho <= 40.
#[allow(dead_code)]
pub fn sigmoid_safe(input: I32F32, rho: I32F32, kappa: I32F32) -> I32F32 {
    let one: I32F32 = I32F32::from_num(1);
    let offset: I32F32 = input.saturating_sub(kappa); // (input - kappa)
    let neg_rho: I32F32 = rho.saturating_mul(one.saturating_neg()); // -rho
    let exp_input: I32F32 = neg_rho.saturating_mul(offset); // -rho*(input-kappa)
    let exp_output: I32F32 = exp_safe(exp_input); // exp(-rho*(input-kappa))
    let denominator: I32F32 = exp_output.saturating_add(one); // 1 + exp(-rho*(input-kappa))
    let sigmoid_output: I32F32 = one.saturating_div(denominator); // 1 / (1 + exp(-rho*(input-kappa)))
    sigmoid_output
}

// Returns a bool vector where an item is true if the vector item is in topk values.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn is_topk(vector: &[I32F32], k: usize) -> Vec<bool> {
    let n: usize = vector.len();
    let mut result: Vec<bool> = vec![true; n];
    if n < k {
        return result;
    }
    let mut idxs: Vec<usize> = (0..n).collect();
    idxs.sort_by_key(|&idx| &vector[idx]); // ascending stable sort
    for &idx in idxs.iter().take(n.saturating_sub(k)) {
        result[idx] = false;
    }
    result
}

// Returns a normalized (sum to 1 except 0) copy of the input vector.
#[allow(dead_code)]
pub fn normalize(x: &[I32F32]) -> Vec<I32F32> {
    let x_sum: I32F32 = sum(x);
    if x_sum != I32F32::from_num(0.0_f32) {
        return x.iter().map(|xi| xi.saturating_div(x_sum)).collect();
    } else {
        x.to_vec()
    }
}

// Normalizes (sum to 1 except 0) the input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize(x: &mut [I32F32]) {
    let x_sum: I32F32 = x.iter().sum();
    if x_sum == I32F32::from_num(0.0_f32) {
        return;
    }
    x.iter_mut()
        .for_each(|value| *value = value.saturating_div(x_sum));
}

// Normalizes (sum to 1 except 0) the input vector directly in-place, using the sum arg.
#[allow(dead_code)]
pub fn inplace_normalize_using_sum(x: &mut [I32F32], x_sum: I32F32) {
    if x_sum == I32F32::from_num(0.0_f32) {
        return;
    }
    x.iter_mut()
        .for_each(|value| *value = value.saturating_div(x_sum));
}

// Normalizes (sum to 1 except 0) the I64F64 input vector directly in-place.
#[allow(dead_code)]
pub fn inplace_normalize_64(x: &mut [I64F64]) {
    let x_sum: I64F64 = x.iter().sum();
    if x_sum == I64F64::from_num(0) {
        return;
    }
    x.iter_mut()
        .for_each(|value| *value = value.saturating_div(x_sum));
}

/// Normalizes (sum to 1 except 0) each row (dim=0) of a I64F64 matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize_64(x: &mut [Vec<I64F64>]) {
    for row in x {
        let row_sum: I64F64 = row.iter().sum();
        if row_sum > I64F64::from_num(0.0_f64) {
            row.iter_mut()
                .for_each(|x_ij: &mut I64F64| *x_ij = x_ij.saturating_div(row_sum));
        }
    }
}

/// Returns x / y for input vectors x and y, if y == 0 return 0.
#[allow(dead_code)]
pub fn vecdiv(x: &[I32F32], y: &[I32F32]) -> Vec<I32F32> {
    assert_eq!(x.len(), y.len());
    x.iter()
        .zip(y)
        .map(|(x_i, y_i)| {
            if *y_i != 0 {
                x_i.saturating_div(*y_i)
            } else {
                I32F32::from_num(0)
            }
        })
        .collect()
}

// Normalizes (sum to 1 except 0) each row (dim=0) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize(x: &mut [Vec<I32F32>]) {
    for row in x {
        let row_sum: I32F32 = row.iter().sum();
        if row_sum > I32F32::from_num(0.0_f32) {
            row.iter_mut()
                .for_each(|x_ij: &mut I32F32| *x_ij = x_ij.saturating_div(row_sum));
        }
    }
}

// Normalizes (sum to 1 except 0) each row (dim=0) of a sparse matrix in-place.
#[allow(dead_code)]
pub fn inplace_row_normalize_sparse(sparse_matrix: &mut [Vec<(u16, I32F32)>]) {
    for sparse_row in sparse_matrix.iter_mut() {
        let row_sum: I32F32 = sparse_row.iter().map(|(_j, value)| *value).sum();
        if row_sum > I32F32::from_num(0.0) {
            sparse_row
                .iter_mut()
                .for_each(|(_j, value)| *value = value.saturating_div(row_sum));
        }
    }
}

// Sum across each row (dim=0) of a matrix.
#[allow(dead_code)]
pub fn row_sum(x: &[Vec<I32F32>]) -> Vec<I32F32> {
    if let Some(first_row) = x.first() {
        if first_row.is_empty() {
            return vec![];
        }
    }
    x.iter().map(|row| row.iter().sum()).collect()
}

// Sum across each row (dim=0) of a sparse matrix.
#[allow(dead_code)]
pub fn row_sum_sparse(sparse_matrix: &[Vec<(u16, I32F32)>]) -> Vec<I32F32> {
    sparse_matrix
        .iter()
        .map(|row| row.iter().map(|(_, value)| value).sum())
        .collect()
}

// Sum across each column (dim=1) of a matrix.
#[allow(dead_code)]
pub fn col_sum(x: &[Vec<I32F32>]) -> Vec<I32F32> {
    let Some(first_row) = x.first() else {
        return vec![];
    };
    let cols = first_row.len();
    if cols == 0 {
        return vec![];
    }
    x.iter()
        .fold(vec![I32F32::from_num(0); cols], |acc, next_row| {
            acc.into_iter()
                .zip(next_row)
                .map(|(acc_elem, next_elem)| acc_elem.saturating_add(*next_elem))
                .collect()
        })
}

// Sum across each column (dim=1) of a sparse matrix.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn col_sum_sparse(sparse_matrix: &[Vec<(u16, I32F32)>], columns: u16) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0); columns as usize];
    for sparse_row in sparse_matrix {
        for (j, value) in sparse_row {
            result[*j as usize] = result[*j as usize].saturating_add(*value);
        }
    }
    result
}

// Normalizes (sum to 1 except 0) each column (dim=1) of a sparse matrix in-place.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn inplace_col_normalize_sparse(sparse_matrix: &mut [Vec<(u16, I32F32)>], columns: u16) {
    let mut col_sum: Vec<I32F32> = vec![I32F32::from_num(0.0); columns as usize]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            col_sum[*j as usize] = col_sum[*j as usize].saturating_add(*value);
        }
    }
    for sparse_row in sparse_matrix {
        for (j, value) in sparse_row {
            if col_sum[*j as usize] == I32F32::from_num(0.0_f32) {
                continue;
            }
            *value = value.saturating_div(col_sum[*j as usize]);
        }
    }
}

// Normalizes (sum to 1 except 0) each column (dim=1) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_normalize(x: &mut [Vec<I32F32>]) {
    let Some(first_row) = x.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    let cols = first_row.len();
    let col_sums = x
        .iter_mut()
        .fold(vec![I32F32::from_num(0.0); cols], |acc, row| {
            row.iter_mut()
                .zip(acc)
                .map(|(&mut m_val, acc_val)| acc_val.saturating_add(m_val))
                .collect()
        });
    x.iter_mut().for_each(|row| {
        row.iter_mut()
            .zip(&col_sums)
            .filter(|(_, col_sum)| **col_sum != I32F32::from_num(0_f32))
            .for_each(|(m_val, col_sum)| {
                *m_val = m_val.saturating_div(*col_sum);
            });
    });
}

// Max-upscale each column (dim=1) of a sparse matrix in-place.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn inplace_col_max_upscale_sparse(sparse_matrix: &mut [Vec<(u16, I32F32)>], columns: u16) {
    let mut col_max: Vec<I32F32> = vec![I32F32::from_num(0.0); columns as usize]; // assume square matrix, rows=cols
    for sparse_row in sparse_matrix.iter() {
        for (j, value) in sparse_row.iter() {
            if col_max[*j as usize] < *value {
                col_max[*j as usize] = *value;
            }
        }
    }
    for sparse_row in sparse_matrix {
        for (j, value) in sparse_row {
            if col_max[*j as usize] == I32F32::from_num(0.0_f32) {
                continue;
            }
            *value = value.saturating_div(col_max[*j as usize]);
        }
    }
}

// Max-upscale each column (dim=1) of a matrix in-place.
#[allow(dead_code)]
pub fn inplace_col_max_upscale(x: &mut [Vec<I32F32>]) {
    let Some(first_row) = x.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    let cols = first_row.len();
    let col_maxes = x
        .iter_mut()
        .fold(vec![I32F32::from_num(0_f32); cols], |acc, row| {
            row.iter_mut()
                .zip(acc)
                .map(|(m_val, acc_val)| acc_val.max(*m_val))
                .collect()
        });
    x.iter_mut().for_each(|row| {
        row.iter_mut()
            .zip(&col_maxes)
            .filter(|(_, col_max)| **col_max != I32F32::from_num(0))
            .for_each(|(m_val, col_max)| {
                *m_val = m_val.saturating_div(*col_max);
            });
    });
}

// Apply mask to vector, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_vector(mask: &[bool], vector: &mut [I32F32]) {
    if mask.is_empty() {
        return;
    }
    assert_eq!(mask.len(), vector.len());
    let zero: I32F32 = I32F32::from_num(0.0);
    mask.iter()
        .zip(vector)
        .filter(|(m, _)| **m)
        .for_each(|(_, v_elem)| {
            *v_elem = zero;
        });
}

// Apply mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_matrix(mask: &[Vec<bool>], matrix: &mut Vec<Vec<I32F32>>) {
    let Some(first_row) = mask.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    assert_eq!(mask.len(), matrix.len());
    let zero: I32F32 = I32F32::from_num(0.0);
    mask.iter().zip(matrix).for_each(|(mask_row, matrix_row)| {
        mask_row
            .iter()
            .zip(matrix_row)
            .filter(|(mask_elem, _)| **mask_elem)
            .for_each(|(_, matrix_elem)| {
                *matrix_elem = zero;
            });
    });
}

// Apply row mask to matrix, mask=true will mask out, i.e. set to 0.
#[allow(dead_code)]
pub fn inplace_mask_rows(mask: &[bool], matrix: &mut [Vec<I32F32>]) {
    let Some(first_row) = matrix.first() else {
        return;
    };
    let cols = first_row.len();
    assert_eq!(mask.len(), matrix.len());
    let zero: I32F32 = I32F32::from_num(0);
    matrix
        .iter_mut()
        .zip(mask)
        .for_each(|(row_elem, mask_row)| {
            if *mask_row {
                *row_elem = vec![zero; cols];
            }
        });
}

// Mask out the diagonal of the input matrix in-place.
#[allow(dead_code)]
pub fn inplace_mask_diag(matrix: &mut [Vec<I32F32>]) {
    let Some(first_row) = matrix.first() else {
        return;
    };
    if first_row.is_empty() {
        return;
    }
    assert_eq!(matrix.len(), first_row.len());
    let zero: I32F32 = I32F32::from_num(0.0);
    matrix.iter_mut().enumerate().for_each(|(idx, row)| {
        let Some(elem) = row.get_mut(idx) else {
            // Should not happen since matrix is square
            return;
        };
        *elem = zero;
    });
}

// Return a new sparse matrix that replaces masked rows with an empty vector placeholder.
#[allow(dead_code)]
pub fn mask_rows_sparse(
    mask: &[bool],
    sparse_matrix: &[Vec<(u16, I32F32)>],
) -> Vec<Vec<(u16, I32F32)>> {
    assert_eq!(sparse_matrix.len(), mask.len());
    mask.iter()
        .zip(sparse_matrix)
        .map(|(mask_elem, sparse_row)| {
            if *mask_elem {
                vec![]
            } else {
                sparse_row.clone()
            }
        })
        .collect()
}

// Return a new sparse matrix with a masked out diagonal of input sparse matrix.
#[allow(dead_code)]
pub fn mask_diag_sparse(sparse_matrix: &[Vec<(u16, I32F32)>]) -> Vec<Vec<(u16, I32F32)>> {
    sparse_matrix
        .iter()
        .enumerate()
        .map(|(i, sparse_row)| {
            sparse_row
                .iter()
                .filter(|(j, _)| i != (*j as usize))
                .copied()
                .collect()
        })
        .collect()
}

// Remove cells from sparse matrix where the mask function of two vectors is true.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn vec_mask_sparse_matrix(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    first_vector: &[u64],
    second_vector: &[u64],
    mask_fn: &dyn Fn(u64, u64) -> bool,
) -> Vec<Vec<(u16, I32F32)>> {
    let n: usize = sparse_matrix.len();
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row {
            if !mask_fn(first_vector[i], second_vector[*j as usize]) {
                result[i].push((*j, *value));
            }
        }
    }
    result
}

// Row-wise matrix-vector hadamard product.
#[allow(dead_code)]
pub fn row_hadamard(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<Vec<I32F32>> {
    let Some(first_row) = matrix.first() else {
        return vec![vec![]];
    };
    if first_row.is_empty() {
        return vec![vec![]];
    }
    matrix
        .iter()
        .zip(vector)
        .map(|(row, vec_val)| {
            row.iter()
                .map(|m_val| vec_val.saturating_mul(*m_val))
                .collect()
        })
        .collect()
}

// Row-wise sparse matrix-vector hadamard product.
#[allow(dead_code)]
pub fn row_hadamard_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
) -> Vec<Vec<(u16, I32F32)>> {
    sparse_matrix
        .iter()
        .zip(vector)
        .map(|(sparse_row, vec_val)| {
            sparse_row
                .iter()
                .map(|(j, value)| (*j, value.saturating_mul(*vec_val)))
                .collect()
        })
        .collect()
}

// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<I32F32> {
    let Some(first_row) = matrix.first() else {
        return vec![];
    };
    let cols = first_row.len();
    if cols == 0 {
        return vec![];
    }
    assert!(matrix.len() == vector.len());
    matrix.iter().zip(vector).fold(
        vec![I32F32::from_num(0_f32); cols],
        |acc, (row, vec_val)| {
            row.iter()
                .zip(acc)
                .map(|(m_val, acc_val)| {
                    // Compute ranks: r_j = SUM(i) w_ij * s_i
                    // Compute trust scores: t_j = SUM(i) w_ij * s_i
                    // result_j = SUM(i) vector_i * matrix_ij
                    acc_val.saturating_add(vec_val.saturating_mul(*m_val))
                })
                .collect()
        },
    )
}

// Row-wise matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code)]
pub fn matmul_64(matrix: &[Vec<I64F64>], vector: &[I64F64]) -> Vec<I64F64> {
    let Some(first_row) = matrix.first() else {
        return vec![];
    };
    let cols = first_row.len();
    if cols == 0 {
        return vec![];
    }
    assert!(matrix.len() == vector.len());
    matrix
        .iter()
        .zip(vector)
        .fold(vec![I64F64::from_num(0.0); cols], |acc, (row, vec_val)| {
            row.iter()
                .zip(acc)
                .map(|(m_val, acc_val)| {
                    // Compute ranks: r_j = SUM(i) w_ij * s_i
                    // Compute trust scores: t_j = SUM(i) w_ij * s_i
                    // result_j = SUM(i) vector_i * matrix_ij
                    acc_val.saturating_add(vec_val.saturating_mul(*m_val))
                })
                .collect()
        })
}

// Column-wise matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code)]
pub fn matmul_transpose(matrix: &[Vec<I32F32>], vector: &[I32F32]) -> Vec<I32F32> {
    let Some(first_row) = matrix.first() else {
        return vec![];
    };
    if first_row.is_empty() {
        return vec![];
    }
    assert!(first_row.len() == vector.len());
    matrix
        .iter()
        .map(|row| {
            row.iter()
                .zip(vector)
                .fold(I32F32::from_num(0.0), |acc, (velem, melem)| {
                    // Compute dividends: d_j = SUM(i) b_ji * inc_i
                    // result_j = SUM(i) vector_i * matrix_ji
                    // result_i = SUM(j) vector_j * matrix_ij
                    acc.saturating_add(velem.saturating_mul(*melem))
                })
        })
        .collect()
}

// Row-wise sparse_matrix-vector product, column-wise sum: result_j = SUM(i) vector_i * matrix_ij.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn matmul_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
    columns: u16,
) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0.0); columns as usize];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute ranks: r_j = SUM(i) w_ij * s_i
            // Compute trust scores: t_j = SUM(i) w_ij * s_i
            // result_j = SUM(i) vector_i * matrix_ij
            result[*j as usize] =
                result[*j as usize].saturating_add(vector[i].saturating_mul(*value));
        }
    }
    result
}

// Column-wise sparse_matrix-vector product, row-wise sum: result_i = SUM(j) vector_j * matrix_ij.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn matmul_transpose_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    vector: &[I32F32],
) -> Vec<I32F32> {
    let mut result: Vec<I32F32> = vec![I32F32::from_num(0.0); sparse_matrix.len()];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            // Compute dividends: d_j = SUM(i) b_ji * inc_i
            // result_j = SUM(i) vector_i * matrix_ji
            // result_i = SUM(j) vector_j * matrix_ij
            result[i] = result[i].saturating_add(vector[*j as usize].saturating_mul(*value));
        }
    }
    result
}

// Set inplace matrix values above column threshold to threshold value.
#[allow(dead_code)]
pub fn inplace_col_clip(x: &mut [Vec<I32F32>], col_threshold: &[I32F32]) {
    x.iter_mut().for_each(|row| {
        row.iter_mut()
            .zip(col_threshold)
            .for_each(|(value, threshold)| {
                *value = *threshold.min(value);
            });
    });
}

// Return sparse matrix with values above column threshold set to threshold value.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn col_clip_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    col_threshold: &[I32F32],
) -> Vec<Vec<(u16, I32F32)>> {
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; sparse_matrix.len()];
    for (i, sparse_row) in sparse_matrix.iter().enumerate() {
        for (j, value) in sparse_row.iter() {
            if col_threshold[*j as usize] < *value {
                if 0 < col_threshold[*j as usize] {
                    result[i].push((*j, col_threshold[*j as usize]));
                }
            } else {
                result[i].push((*j, *value));
            }
        }
    }
    result
}

// Set matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn clip(
    x: &[Vec<I32F32>],
    threshold: I32F32,
    upper: I32F32,
    lower: I32F32,
) -> Vec<Vec<I32F32>> {
    x.iter()
        .map(|row| {
            row.iter()
                .map(|elem| if *elem >= threshold { upper } else { lower })
                .collect()
        })
        .collect()
}

// Set inplace matrix values below threshold to lower, and equal-above to upper.
#[allow(dead_code)]
pub fn inplace_clip(x: &mut [Vec<I32F32>], threshold: I32F32, upper: I32F32, lower: I32F32) {
    x.iter_mut().for_each(|row| {
        row.iter_mut().for_each(|elem| {
            *elem = if *elem >= threshold { upper } else { lower };
        });
    });
}

// Set sparse matrix values below threshold to lower, and equal-above to upper.
// Does not add missing elements (0 value assumed) when lower!=0.
#[allow(dead_code)]
pub fn clip_sparse(
    sparse_matrix: &[Vec<(u16, I32F32)>],
    threshold: I32F32,
    upper: I32F32,
    lower: I32F32,
) -> Vec<Vec<(u16, I32F32)>> {
    sparse_matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|(j, value)| {
                    if *value < threshold {
                        (*j, lower)
                    } else {
                        (*j, upper)
                    }
                })
                .collect()
        })
        .collect()
}

// Stake-weighted median score finding algorithm, based on a mid pivot binary search.
// Normally a random pivot is used, but to ensure full determinism the mid point is chosen instead.
// Assumes relatively random score order for efficiency, typically less than O(nlogn) complexity.
//
// # Args:
// 	* 'stake': ( &[I32F32] ):
//         - stake, assumed to be normalized.
//
// 	* 'score': ( &[I32F32] ):
//         - score for which median is sought, 0 <= score <= 1
//
// 	* 'partition_idx' ( &[usize] ):
// 		- indices as input partition
//
// 	* 'minority' ( I32F32 ):
// 		- minority_ratio = 1 - majority_ratio
//
// 	* 'partition_lo' ( I32F32 ):
// 		- lower edge of stake for partition, where partition is a segment [lo, hi] inside stake integral [0, 1].
//
// 	* 'partition_hi' ( I32F32 ):
// 		- higher edge of stake for partition, where partition is a segment [lo, hi] inside stake integral [0, 1].
//
// # Returns:
//     * 'median': ( I32F32 ):
//         - median via random pivot binary search.
//
#[allow(dead_code, clippy::indexing_slicing)]
pub fn weighted_median(
    stake: &[I32F32],
    score: &[I32F32],
    partition_idx: &[usize],
    minority: I32F32,
    partition_lo: I32F32,
    partition_hi: I32F32,
) -> I32F32 {
    let n = partition_idx.len();
    if n == 0 {
        return I32F32::from_num(0);
    }
    if n == 1 {
        return score[partition_idx[0]];
    }
    assert!(stake.len() == score.len());
    let mid_idx: usize = n.saturating_div(2);
    let pivot: I32F32 = score[partition_idx[mid_idx]];
    let mut lo_stake: I32F32 = I32F32::from_num(0);
    let mut hi_stake: I32F32 = I32F32::from_num(0);
    let mut lower: Vec<usize> = vec![];
    let mut upper: Vec<usize> = vec![];
    for &idx in partition_idx {
        if score[idx] == pivot {
            continue;
        }
        if score[idx] < pivot {
            lo_stake = lo_stake.saturating_add(stake[idx]);
            lower.push(idx);
        } else {
            hi_stake = hi_stake.saturating_add(stake[idx]);
            upper.push(idx);
        }
    }
    if (partition_lo.saturating_add(lo_stake) <= minority)
        && (minority < partition_hi.saturating_sub(hi_stake))
    {
        return pivot;
    } else if (minority < partition_lo.saturating_add(lo_stake)) && (!lower.is_empty()) {
        return weighted_median(
            stake,
            score,
            &lower,
            minority,
            partition_lo,
            partition_lo.saturating_add(lo_stake),
        );
    } else if (partition_hi.saturating_sub(hi_stake) <= minority) && (!upper.is_empty()) {
        return weighted_median(
            stake,
            score,
            &upper,
            minority,
            partition_hi.saturating_sub(hi_stake),
            partition_hi,
        );
    }
    pivot
}

/// Column-wise weighted median, e.g. stake-weighted median scores per server (column) over all validators (rows).
#[allow(dead_code, clippy::indexing_slicing)]
pub fn weighted_median_col(
    stake: &[I32F32],
    score: &[Vec<I32F32>],
    majority: I32F32,
) -> Vec<I32F32> {
    let rows = stake.len();
    let columns = score[0].len();
    let zero: I32F32 = I32F32::from_num(0);
    let mut median: Vec<I32F32> = vec![zero; columns];

    #[allow(clippy::needless_range_loop)]
    for c in 0..columns {
        let mut use_stake: Vec<I32F32> = vec![];
        let mut use_score: Vec<I32F32> = vec![];
        for r in 0..rows {
            assert_eq!(columns, score[r].len());
            if stake[r] > zero {
                use_stake.push(stake[r]);
                use_score.push(score[r][c]);
            }
        }
        if !use_stake.is_empty() {
            inplace_normalize(&mut use_stake);
            let stake_sum: I32F32 = use_stake.iter().sum();
            let minority: I32F32 = stake_sum.saturating_sub(majority);
            median[c] = weighted_median(
                &use_stake,
                &use_score,
                (0..use_stake.len()).collect::<Vec<_>>().as_slice(),
                minority,
                zero,
                stake_sum,
            );
        }
    }
    median
}

/// Column-wise weighted median, e.g. stake-weighted median scores per server (column) over all validators (rows).
#[allow(dead_code, clippy::indexing_slicing)]
pub fn weighted_median_col_sparse(
    stake: &[I32F32],
    score: &[Vec<(u16, I32F32)>],
    columns: u16,
    majority: I32F32,
) -> Vec<I32F32> {
    let rows = stake.len();
    let zero: I32F32 = I32F32::from_num(0);
    let mut use_stake: Vec<I32F32> = stake.iter().copied().filter(|&s| s > zero).collect();
    inplace_normalize(&mut use_stake);
    let stake_sum: I32F32 = use_stake.iter().sum();
    let stake_idx: Vec<usize> = (0..use_stake.len()).collect();
    let minority: I32F32 = stake_sum.saturating_sub(majority);
    let mut use_score: Vec<Vec<I32F32>> = vec![vec![zero; use_stake.len()]; columns as usize];
    let mut median: Vec<I32F32> = vec![zero; columns as usize];
    let mut k: usize = 0;
    for r in 0..rows {
        if stake[r] <= zero {
            continue;
        }
        for (c, val) in score[r].iter() {
            use_score[*c as usize][k] = *val;
        }
        k.saturating_inc();
    }
    for c in 0..columns as usize {
        median[c] = weighted_median(
            &use_stake,
            &use_score[c],
            &stake_idx,
            minority,
            zero,
            stake_sum,
        );
    }
    median
}

// Element-wise product of two matrices.
#[allow(dead_code)]
pub fn hadamard(mat1: &[Vec<I32F32>], mat2: &[Vec<I32F32>]) -> Vec<Vec<I32F32>> {
    assert!(mat1.len() == mat2.len());
    let Some(first_row) = mat1.first() else {
        return vec![vec![]];
    };
    if first_row.is_empty() {
        return vec![vec![]];
    }
    mat1.iter()
        .zip(mat2)
        .map(|(row1, row2)| {
            assert!(row1.len() == row2.len());
            row1.iter()
                .zip(row2)
                .map(|(elem1, elem2)| elem1.saturating_mul(*elem2))
                .collect()
        })
        .collect()
}

// Element-wise product of two sparse matrices.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn hadamard_sparse(
    mat1: &[Vec<(u16, I32F32)>],
    mat2: &[Vec<(u16, I32F32)>],
    columns: u16,
) -> Vec<Vec<(u16, I32F32)>> {
    assert!(mat1.len() == mat2.len());
    let rows = mat1.len();
    let zero: I32F32 = I32F32::from_num(0);
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; rows];
    for i in 0..rows {
        let mut row1: Vec<I32F32> = vec![zero; columns as usize];
        for (j, value) in mat1[i].iter() {
            row1[*j as usize] = row1[*j as usize].saturating_add(*value);
        }
        let mut row2: Vec<I32F32> = vec![zero; columns as usize];
        for (j, value) in mat2[i].iter() {
            row2[*j as usize] = row2[*j as usize].saturating_add(*value);
        }
        for j in 0..columns as usize {
            let prod: I32F32 = row1[j].saturating_mul(row2[j]);
            if zero < prod {
                result[i].push((j as u16, prod))
            }
        }
    }
    result
}

// Return matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
// `alpha` is the EMA coefficient, how much to add of the new observation, typically small,
// higher alpha discounts older observations faster.
#[allow(dead_code)]
pub fn mat_ema(new: &[Vec<I32F32>], old: &[Vec<I32F32>], alpha: I32F32) -> Vec<Vec<I32F32>> {
    let Some(first_row) = new.first() else {
        return vec![vec![]];
    };
    if first_row.is_empty() {
        return vec![vec![]; 1];
    }
    let one_minus_alpha: I32F32 = I32F32::from_num(1.0).saturating_sub(alpha);
    new.iter()
        .zip(old)
        .map(|(new_row, old_row)| {
            new_row
                .iter()
                .zip(old_row)
                .map(|(new_elem, old_elem)| {
                    alpha
                        .saturating_mul(*new_elem)
                        .saturating_add(one_minus_alpha.saturating_mul(*old_elem))
                })
                .collect()
        })
        .collect()
}

// Return sparse matrix exponential moving average: `alpha * a_ij + one_minus_alpha * b_ij`.
// `alpha` is the EMA coefficient, how much to add of the new observation, typically small,
// higher alpha discounts older observations faster.
#[allow(dead_code, clippy::indexing_slicing)]
pub fn mat_ema_sparse(
    new: &[Vec<(u16, I32F32)>],
    old: &[Vec<(u16, I32F32)>],
    alpha: I32F32,
) -> Vec<Vec<(u16, I32F32)>> {
    assert!(new.len() == old.len());
    let n = new.len(); // assume square matrix, rows=cols
    let zero: I32F32 = I32F32::from_num(0.0);
    let one_minus_alpha: I32F32 = I32F32::from_num(1.0).saturating_sub(alpha);
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
    for i in 0..new.len() {
        let mut row: Vec<I32F32> = vec![zero; n];
        for (j, value) in new[i].iter() {
            row[*j as usize] = row[*j as usize].saturating_add(alpha.saturating_mul(*value));
        }
        for (j, value) in old[i].iter() {
            row[*j as usize] =
                row[*j as usize].saturating_add(one_minus_alpha.saturating_mul(*value));
        }
        for (j, value) in row.iter().enumerate() {
            if *value > zero {
                result[i].push((j as u16, *value))
            }
        }
    }
    result
}

// Return sparse matrix only with elements >= threshold of an input sparse matrix.
#[allow(dead_code)]
pub fn sparse_threshold(w: &[Vec<(u16, I32F32)>], threshold: I32F32) -> Vec<Vec<(u16, I32F32)>> {
    w.iter()
        .map(|row| {
            row.iter()
                .filter(|(_, weight)| *weight >= threshold)
                .copied()
                .collect()
        })
        .collect()
}

/// Calculates the exponential moving average (EMA) for a sparse matrix using dynamic alpha values.
#[allow(dead_code)]
pub fn mat_ema_alpha_vec_sparse(
    new: &[Vec<(u16, I32F32)>],
    old: &[Vec<(u16, I32F32)>],
    alpha: &[I32F32],
) -> Vec<Vec<(u16, I32F32)>> {
    // Ensure the new and old matrices have the same number of rows.
    assert!(new.len() == old.len());
    let n = new.len(); // Assume square matrix, rows=cols
    let zero: I32F32 = I32F32::from_num(0.0);
    let mut result: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];

    // Iterate over each row of the matrices.
    for (i, (new_row, old_row)) in new.iter().zip(old).enumerate() {
        // Initialize a row of zeros for the result matrix.
        let mut row: Vec<I32F32> = vec![zero; n];

        // Process the new matrix values.
        for (j, value) in new_row.iter() {
            // Retrieve the alpha value for the current column.
            let alpha_val: I32F32 = alpha.get(*j as usize).copied().unwrap_or(zero);
            // Compute the EMA component for the new value using saturating multiplication.
            if let Some(row_val) = row.get_mut(*j as usize) {
                *row_val = alpha_val.saturating_mul(*value);
            }
            log::trace!(
                "new[{}][{}] * alpha[{}] = {} * {} = {}",
                i,
                j,
                j,
                value,
                alpha_val,
                row.get(*j as usize).unwrap_or(&zero)
            );
        }

        // Process the old matrix values.
        for (j, value) in old_row.iter() {
            // Retrieve the alpha value for the current column.
            let alpha_val: I32F32 = alpha.get(*j as usize).copied().unwrap_or(zero);
            // Calculate the complement of the alpha value using saturating subtraction.
            let one_minus_alpha: I32F32 = I32F32::from_num(1.0).saturating_sub(alpha_val);
            // Compute the EMA component for the old value and add it to the row using saturating operations.
            if let Some(row_val) = row.get_mut(*j as usize) {
                *row_val = row_val.saturating_add(one_minus_alpha.saturating_mul(*value));
            }
            log::trace!(
                "old[{}][{}] * (1 - alpha[{}]) = {} * {} = {}",
                i,
                j,
                j,
                value,
                one_minus_alpha,
                one_minus_alpha.saturating_mul(*value)
            );
        }

        // Collect the non-zero values into the result matrix.
        for (j, value) in row.iter().enumerate() {
            if *value > zero {
                if let Some(result_row) = result.get_mut(i) {
                    result_row.push((j as u16, *value));
                    log::trace!("result[{}][{}] = {}", i, j, value);
                }
            }
        }
    }

    // Return the computed EMA sparse matrix.
    result
}

/// Return matrix exponential moving average: `alpha_j * a_ij + one_minus_alpha_j * b_ij`.
/// `alpha_` is the EMA coefficient passed as a vector per column.
#[allow(dead_code)]
pub fn mat_ema_alpha_vec(
    new: &[Vec<I32F32>],
    old: &[Vec<I32F32>],
    alpha: &[I32F32],
) -> Vec<Vec<I32F32>> {
    // Check if the new matrix is empty or its first row is empty.
    if new.is_empty() || new.first().map_or(true, |row| row.is_empty()) {
        return vec![vec![]; 1];
    }

    // Ensure the dimensions of the new and old matrices match.
    assert!(new.len() == old.len());
    assert!(new.first().map_or(0, |row| row.len()) == alpha.len());

    // Initialize the result matrix with zeros, having the same dimensions as the new matrix.
    let mut result: Vec<Vec<I32F32>> =
        vec![vec![I32F32::from_num(0.0); new.first().map_or(0, |row| row.len())]; new.len()];

    // Iterate over each row of the matrices.
    for (i, (new_row, old_row)) in new.iter().zip(old).enumerate() {
        // Ensure the current row of the new and old matrices have the same length.
        assert!(new_row.len() == old_row.len());

        // Iterate over each column of the current row.
        for (j, &alpha_val) in alpha.iter().enumerate().take(new_row.len()) {
            // Calculate the complement of the alpha value using saturating subtraction.
            let one_minus_alpha = I32F32::from_num(1.0).saturating_sub(alpha_val);

            // Compute the EMA for the current element using saturating operations.
            if let (Some(new_val), Some(old_val), Some(result_val)) = (
                new_row.get(j),
                old_row.get(j),
                result.get_mut(i).and_then(|row| row.get_mut(j)),
            ) {
                *result_val = alpha_val
                    .saturating_mul(*new_val)
                    .saturating_add(one_minus_alpha.saturating_mul(*old_val));
            }
        }
    }

    // Return the computed EMA matrix.
    result
}

/// Return the quantile of a vector of I32F32 values.
pub fn quantile(data: &[I32F32], quantile: f64) -> I32F32 {
    // Clone the input data to avoid modifying the original vector.
    let mut sorted_data = data.to_owned();

    // Sort the cloned data in ascending order, handling potential NaN values.
    sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    // Get the length of the sorted data.
    let len = sorted_data.len();

    // If the data is empty, return 0 as the quantile value.
    if len == 0 {
        return I32F32::from_num(0);
    }

    // Calculate the position in the sorted array corresponding to the quantile.
    let pos = quantile * (len.saturating_sub(1)) as f64;

    // Determine the lower index by flooring the position.
    let low = pos.floor() as usize;

    // Determine the higher index by ceiling the position.
    let high = pos.ceil() as usize;

    // If the low and high indices are the same, return the value at that index.
    if low == high {
        sorted_data
            .get(low)
            .copied()
            .unwrap_or_else(|| I32F32::from_num(0))
    } else {
        // Otherwise, perform linear interpolation between the low and high values.
        let low_value = sorted_data
            .get(low)
            .copied()
            .unwrap_or_else(|| I32F32::from_num(0));
        let high_value = sorted_data
            .get(high)
            .copied()
            .unwrap_or_else(|| I32F32::from_num(0));

        // Calculate the weight for interpolation.
        let weight = I32F32::from_num(pos - low as f64);

        // Return the interpolated value using saturating operations.
        low_value.saturating_add((high_value.saturating_sub(low_value)).saturating_mul(weight))
    }
}

/// Safe ln function, returns 0 if value is 0.
pub fn safe_ln(value: I32F32) -> I32F32 {
    ln(value).unwrap_or(I32F32::from_num(0.0))
}

/// Safe exp function, returns 0 if value is 0.
pub fn safe_exp(value: I32F32) -> I32F32 {
    exp(value).unwrap_or(I32F32::from_num(0.0))
}
