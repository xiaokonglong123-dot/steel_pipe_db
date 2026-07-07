use sqlx::SqlitePool;

use crate::domain::pipe::PipeModel;
use crate::error::AppError;
use crate::repositories::generic_pipe_repo::GenericPipeRepo;

/// Common pipe operations — deduplicated helper functions for pipe validations.
/// Only contains `validate_pipes_for_inbound` using GenericPipeRepo (no raw SQL).
pub struct PipeHelpers;

impl PipeHelpers {
    pub async fn validate_pipes_for_inbound<P: PipeModel>(
        pool: &SqlitePool,
        pipe_ids: &[i64],
    ) -> Result<(), AppError> {
        let pipes = GenericPipeRepo::<P>::find_by_ids(pool, pipe_ids)
            .await
            .map_err(AppError::from)?;
        let found_ids: std::collections::HashSet<i64> =
            pipes.iter().map(|p| p.id()).collect();
        for id in pipe_ids {
            if !found_ids.contains(id) {
                return Err(AppError::NotFound(format!(
                    "Pipe id={} not found or has been deleted",
                    id
                )));
            }
        }
        for pipe in &pipes {
            if pipe.status() == "in_stock" {
                return Err(AppError::Validation(format!(
                    "Pipe id={} (pipe_number={}) is already in_stock, cannot inbound again",
                    pipe.id(), pipe.pipe_number()
                )));
            }
        }
        Ok(())
    }
}
