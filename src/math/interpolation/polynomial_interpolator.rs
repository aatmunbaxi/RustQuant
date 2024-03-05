// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// RustQuant: A Rust library for quantitative finance tools.
// Copyright (C) 2023 https://github.com/avhz
// Dual licensed under Apache 2.0 and MIT.
// See:
//      - LICENSE-APACHE.md
//      - LICENSE-MIT.md
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

//! Module containing functionality for polynomial Lagrance interpolation.
//!
//! Intepolates a set of points in the plane with Lagrange polynomials
//! with the barycentric method described by Berrut and Trefethen
//! in "Barycentric Lagrance Interpolation"

use crate::math::interpolation::{
    InterpolationError, InterpolationIndex, InterpolationValue, Interpolator,
};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// STRUCTS & ENUMS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Polynomial Interpolator using the barycentric method
pub struct PolynomialInterpolator<IndexType, ValueType>
where
    IndexType: InterpolationIndex,
    ValueType: InterpolationValue,
{
    /// X-axis values for the interpolator.
    pub xs: Vec<IndexType>,

    /// Y-axis values for the interpolator.
    pub ys: Vec<ValueType>,

    /// Barycentric weights
    pub bary_weights: Vec<ValueType>,

    /// Whether the interpolator has been fitted.
    pub fitted: bool,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// IMPLEMENTATIONS, FUNCTIONS, AND MACROS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

impl<IndexType, ValueType> PolynomialInterpolator<IndexType, ValueType>
where
    IndexType: InterpolationIndex,
    ValueType: InterpolationValue,
{
    /// Create a new PolynomialInterpolator.
    ///
    /// # Errors
    /// - `InterpolationError::UnequalLength` if ```xs.length() != ys.length()```.
    ///
    /// # Panics
    /// Panics if NaN is in the index.
    pub fn new(
        xs: Vec<IndexType>,
        ys: Vec<ValueType>,
    ) -> Result<PolynomialInterpolator<IndexType, ValueType>, InterpolationError> {
        if xs.len() != ys.len() {
            return Err(InterpolationError::UnequalLength);
        }

        let mut tmp: Vec<_> = xs.into_iter().zip(ys).collect();

        tmp.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let (xs, ys): (Vec<IndexType>, Vec<ValueType>) = tmp.into_iter().unzip();

        Ok(Self {
            xs,
            ys,
            fitted: false,
        })
    }
}

impl<IndexType, ValueType> Interpolator<IndexType, ValueType>
    for PolynomialInterpolator<IndexType, ValueType>
where
    IndexType: InterpolationIndex<DeltaDiv = ValueType>,
    ValueType: InterpolationValue,
{
    fn fit(&mut self) -> Result<(), InterpolationError> {
        self.fitted = true;
        Ok(())
    }

    fn range(&self) -> (IndexType, IndexType) {
        (*self.xs.first().unwrap(), *self.xs.last().unwrap())
    }

    fn add_point(&mut self, point: (IndexType, ValueType)) {
        let idx = self.xs.partition_point(|&x| x < point.0);
        self.xs.insert(idx, point.0);
        self.ys.insert(idx, point.1);
    }

    fn interpolate(&self, point: IndexType) -> Result<ValueType, InterpolationError> {
        let range = self.range();
        if point.partial_cmp(&range.0).unwrap() == std::cmp::Ordering::Less
            || point.partial_cmp(&range.1).unwrap() == std::cmp::Ordering::Greater
        {
            return Err(InterpolationError::OutsideOfRange);
        }
        if let Ok(idx) = self
            .xs
            .binary_search_by(|p| p.partial_cmp(&point).expect("Cannot compare values."))
        {
            return Ok(self.ys[idx]);
        }
        let idx_r = self.xs.partition_point(|&x| x < point);
        let idx_l = idx_r - 1;

        let term_1 = self.ys[idx_r] - self.ys[idx_l];
        let term_2 = (point - self.xs[idx_l]) / (self.xs[idx_r] - self.xs[idx_l]);

        let result = self.ys[idx_l] + term_1 * term_2;

        Ok(result)
    }
}
impl<IndexType, ValueType> Interpolator<IndexType, ValueType>
    for PolynomialInterpolator<IndexType, ValueType>
where
    IndexType: InterpolationIndex<DeltaDiv = ValueType>,
    ValueType: InterpolationValue,
{
    fn compute_barycentric_weights(self) {
        todo!()
    }
}
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Unit tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(test)]
mod tests_linear_interpolation {
    use super::*;
    use crate::{assert_approx_equal, RUSTQUANT_EPSILON};
    use time::macros::date;

    #[test]
    fn test_linear_interpolation() {
        let xs = vec![1., 2., 3., 4., 5.];
        let ys = vec![1., 2., 3., 4., 5.];

        let mut interpolator = PolynomialInterpolator::new(xs, ys).unwrap();
        let _ = interpolator.fit();

        assert_approx_equal!(
            2.5,
            interpolator.interpolate(2.5).unwrap(),
            RUSTQUANT_EPSILON
        );
        assert_approx_equal!(
            3.5,
            interpolator.interpolate(3.5).unwrap(),
            RUSTQUANT_EPSILON
        );
    }

    #[test]
    fn test_linear_interpolation_out_of_range() {
        let xs = vec![1., 2., 3., 4., 5.];
        let ys = vec![1., 2., 3., 4., 5.];

        let mut interpolator = PolynomialInterpolator::new(xs, ys).unwrap();
        let _ = interpolator.fit();

        assert!(InterpolationError::OutsideOfRange == interpolator.interpolate(6.).err().unwrap());
    }

    #[test]
    fn test_linear_interpolation_dates() {
        let now = time::OffsetDateTime::now_utc();

        let xs = vec![
            now,
            now + time::Duration::days(1),
            now + time::Duration::days(2),
            now + time::Duration::days(3),
            now + time::Duration::days(4),
        ];

        let ys = vec![1., 2., 3., 4., 5.];

        let mut interpolator = PolynomialInterpolator::new(xs.clone(), ys).unwrap();
        let _ = interpolator.fit();

        assert_approx_equal!(
            2.5,
            interpolator
                .interpolate(xs[1] + time::Duration::hours(12))
                .unwrap(),
            RUSTQUANT_EPSILON
        );
    }

    #[test]
    fn test_linear_interpolation_dates_textbook() {
        let d_1m = date!(1990 - 06 - 16);
        let d_2m = date!(1990 - 07 - 17);

        let r_1m = 0.9870;
        let r_2m = 0.9753;

        let dates = vec![d_1m, d_2m];
        let rates = vec![r_1m, r_2m];

        let mut interpolator = PolynomialInterpolator::new(dates, rates).unwrap();

        assert_approx_equal!(
            0.9855,
            interpolator.interpolate(date!(1990 - 06 - 20)).unwrap(),
            RUSTQUANT_EPSILON
        );
    }
}
