//! Prior
//!
//! When asked to predict a value for an input that is too dissimilar to known inputs, the model will return the prior.
//! Furthermore the process will be fitted on the residual of the prior meaning that a good prior can significantly improve the precision of the model.
//!
//! This can be a constant but also a polynomial or any model of the data.
//! If you want to provide a user-defined prior, it should implement the Prior trait.

use nalgebra::DVector;
use nalgebra::{storage::Storage, U1, Dynamic};
use crate::algebra::{SVector, SMatrix};

//---------------------------------------------------------------------------------------
// TRAIT

/// The Prior trait
///
/// If you want to provide a user-defined kernel, you should implement this trait.
pub trait Prior
{
   /// Default value for the prior
   fn default(input_dimension: usize) -> Self;

   /// Takes and input and return an output.
   fn prior<S: Storage<f64, Dynamic, Dynamic>>(&self, input: &SMatrix<S>) -> DVector<f64>;

   /// Optional, function that fits the prior on training data
   fn fit<SM: Storage<f64, Dynamic, Dynamic> + Clone, SV: Storage<f64, Dynamic, U1>>(&mut self,
                                                                                     _training_inputs: &SMatrix<SM>,
                                                                                     _training_outputs: &SVector<SV>)
   {
   }
}

//---------------------------------------------------------------------------------------
// CLASSICAL PRIOR

/// The Zero prior
///
/// this prior always return zero.
#[derive(Clone, Copy, Debug)]
pub struct Zero {}

impl Prior for Zero
{
   fn default(_input_dimension: usize) -> Self
   {
      Zero {}
   }

   fn prior<S: Storage<f64, Dynamic, Dynamic>>(&self, input: &SMatrix<S>) -> DVector<f64>
   {
      DVector::zeros(input.nrows())
   }
}

//-----------------------------------------------

/// The Constant prior
///
/// This prior returns a constant.
/// It can be fit to return the mean of the training data.
#[derive(Clone, Debug)]
pub struct Constant
{
   c: f64
}

impl Constant
{
   /// Constructs a new constant prior
   pub fn new(c: f64) -> Constant
   {
      Constant { c: c }
   }
}

impl Prior for Constant
{
   fn default(_input_dimension: usize) -> Constant
   {
      Constant::new(0f64)
   }

   fn prior<S: Storage<f64, Dynamic, Dynamic>>(&self, input: &SMatrix<S>) -> DVector<f64>
   {
      DVector::from_element(input.nrows(), self.c)
   }

   /// the prior is fitted on the mean of the training outputs
   fn fit<SM: Storage<f64, Dynamic, Dynamic>, SV: Storage<f64, Dynamic, U1>>(&mut self,
                                                                             _training_inputs: &SMatrix<SM>,
                                                                             training_outputs: &SVector<SV>)
   {
      self.c = training_outputs.mean();
   }
}

//-----------------------------------------------

/// The Linear prior
///
/// This prior is a linear function which can be fit on the training data.
#[derive(Clone, Debug)]
pub struct Linear
{
   weights: DVector<f64>,
   intercept: f64
}

impl Linear
{
   /// Constructs a new linear prior
   /// the first row of w is the bias such that `prior = [1|input] * w`
   pub fn new(weights: DVector<f64>, intercept: f64) -> Self
   {
      Linear { weights, intercept }
   }
}

impl Prior for Linear
{
   fn default(input_dimension: usize) -> Linear
   {
      Linear { weights: DVector::zeros(input_dimension), intercept: 0f64 }
   }

   fn prior<S: Storage<f64, Dynamic, Dynamic>>(&self, input: &SMatrix<S>) -> DVector<f64>
   {
      let mut result = input * &self.weights;
      result.add_scalar_mut(self.intercept);
      result
   }

   /// performs a linear fit to set the value of the prior
   fn fit<SM: Storage<f64, Dynamic, Dynamic> + Clone, SV: Storage<f64, Dynamic, U1>>(&mut self,
                                                                                     training_inputs: &SMatrix<SM>,
                                                                                     training_outputs: &SVector<SV>)
   {
      // solve linear system using LU decomposition
      let weights = training_inputs.clone()
                                   .insert_column(0, 1f64) // add constant term for non-zero intercept
                                   .lu()
                                   .solve(training_outputs)
                                   .expect("Resolution of linear system failed");
      // extracts weights and intercept
      self.intercept = weights[0];
      self.weights = weights.remove_row(0);
   }
}
