use std::{str::FromStr, sync::Arc, time::Instant};

use error_stack::ResultExt;
use tokio::task::JoinSet;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::{
    AppResult,
    errors::Error,
    output::{
        db::{pg::Pg, sync_data_task::SyncDataTaskDao},
        plugins::{get_registry, list_plugin_names},
    },
};

pub struct Scheduler {
    sched: JobScheduler,
    pg: Arc<Pg>,
}

impl Scheduler {
    pub async fn try_new(pg: Pg) -> AppResult<Self> {
        let sched = JobScheduler::new()
            .await
            .change_context_lazy(|| Error::Message("Failed to create scheduler".to_string()))?;
        Ok(Scheduler {
            sched,
            pg: Arc::new(pg),
        })
    }

    fn create_job(&self, interval_minutes: i32) -> AppResult<tokio_cron_scheduler::Job> {
        let cron_syntax = format!("0 */{} * * * *", interval_minutes);
        log::debug!("Creating job with cron syntax: {}", cron_syntax);
        let job =
            tokio_cron_scheduler::Job::new_async(cron_syntax.as_str(), move |uuid, mut _l| {
                Box::pin(async move {
                    execute_job(uuid).await;
                })
            })
            .change_context_lazy(|| {
                Error::Message("Failed to create new job for task".to_string())
            })?;
        Ok(job)
    }

    async fn add_job_and_update_db(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        task: &crate::domain::models::sync_data_task::SyncDataTask,
        job: tokio_cron_scheduler::Job,
    ) -> AppResult<()> {
        let new_job_id = self.sched.add(job).await.change_context_lazy(|| {
            Error::Message("Failed to add new job to scheduler".to_string())
        })?;
        SyncDataTaskDao::update_job(tx, task.id, new_job_id.to_string()).await?;
        Ok(())
    }

    async fn remove_existing_job(&self, job_id: &str) -> AppResult<Option<Uuid>> {
        let job_id = Uuid::from_str(job_id)
            .change_context_lazy(|| Error::Message("Failed to parse job ID".to_string()))?;
        self.sched.remove(&job_id).await.change_context_lazy(|| {
            Error::Message(format!(
                "Failed to remove existing job {} from scheduler",
                job_id
            ))
        })?;
        log::info!("Removed existing job with UUID: {}", job_id);
        Ok(Some(job_id))
    }

    pub async fn update(&self, id: i64) -> AppResult<()> {
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        let sync_data_task = SyncDataTaskDao::first(&mut tx).await?;
        if let Some(task) = sync_data_task {
            if let Some(job_id) = &task.job_id {
                self.remove_existing_job(job_id).await?;
            }

            let job = self.create_job(task.interval_minutes)?;
            self.add_job_and_update_db(&mut tx, &task, job).await?;
            tx.commit().await.change_context_lazy(|| {
                Error::Message("Failed to commit transaction".to_string())
            })?;
            Ok(())
        } else {
            log::error!("Failed to find scheduled task in database by id {}", id);
            Err(
                Error::Message("Failed to find scheduled task in database by id".to_string())
                    .into(),
            )
        }
    }

    pub async fn init_from_db(self) -> AppResult<Self> {
        let mut tx =
            self.pg.pool.begin().await.change_context_lazy(|| {
                Error::Message("failed to begin transaction".to_string())
            })?;
        if let Some(task) = SyncDataTaskDao::first(&mut tx).await? {
            log::info!(
                "Found scheduled task: name={}, minute={}, status={}",
                task.name,
                task.interval_minutes,
                task.status
            );

            if task.status {
                let job = self.create_job(task.interval_minutes)?;
                self.add_job_and_update_db(&mut tx, &task, job).await?;
            } else {
                log::info!("Scheduled task '{}' is disabled", task.name);
            }
        } else {
            log::info!("No scheduled tasks found in database");
        }
        self.sched
            .start()
            .await
            .change_context_lazy(|| Error::Message("Failed to start scheduler".to_string()))?;
        tx.commit()
            .await
            .change_context_lazy(|| Error::Message("Failed to commit transaction".to_string()))?;
        Ok(self)
    }
}

async fn execute_job(_uuid: Uuid) {
    log::info!("Executing scheduled job...");
    let start = Instant::now();
    let mut job_set = JoinSet::new();
    let plugin_names = list_plugin_names();

    for plugin_name in plugin_names {
        job_set.spawn(async move {
            let plugins = get_registry();
            log::info!("Updating plugin: {}", plugin_name);
            if let Some(plugin) = plugins.get::<str>(&plugin_name)
                && let Err(e) = plugin.update(1).await
            {
                log::error!("Plugin update failed for {}: {}", plugin_name, e)
            }
        });
    }
    job_set.join_all().await;
    log::info!("Plugin syn finished elapsed {:?}", start.elapsed());
}
