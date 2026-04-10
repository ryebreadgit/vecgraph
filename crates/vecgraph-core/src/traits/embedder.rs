use crate::error::VecGraphError;

pub trait Embedder: Send + Sync {
    fn embed(&self, input: &str) -> Result<Vec<f32>, VecGraphError>;
    fn dimensions(&self) -> usize;
    fn embed_batch(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>, VecGraphError> {
        inputs.iter().map(|input| self.embed(input)).collect()
    }
    fn arithmetic(
        &self,
        base: &[f32],
        add: &[&[f32]],
        sub: &[&[f32]],
    ) -> Result<Vec<f32>, VecGraphError> {
        let dim = self.dimensions();
        if base.len() != dim {
            return Err(VecGraphError::DimensionMismatch {
                expected: dim,
                got: base.len(),
            });
        }

        let mut result = base.to_vec();

        // Simple vector arithmetic: result = base + sum(add) - sum(sub)

        for v in add {
            if v.len() != dim {
                return Err(VecGraphError::DimensionMismatch {
                    expected: dim,
                    got: v.len(),
                });
            }
            for (r, val) in result.iter_mut().zip(v.iter()) {
                *r += val;
            }
        }

        for v in sub {
            if v.len() != dim {
                return Err(VecGraphError::DimensionMismatch {
                    expected: dim,
                    got: v.len(),
                });
            }
            for (r, val) in result.iter_mut().zip(v.iter()) {
                *r -= val;
            }
        }

        // Normalize the result to unit length
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in result.iter_mut() {
                *x /= norm;
            }
        }

        Ok(result)
    }
}
