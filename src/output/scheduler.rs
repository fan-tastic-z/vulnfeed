use std::{sync::Arc, time::Instant};

use error_stack::{Result, ResultExt};
use tokio::task::JoinSet;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::{
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
    pub async fn try_new(pg: Pg) -> Result<Self, Error> {
        let sched = JobScheduler::new()
            .await
            .change_context_lazy(|| Error::Message("Failed to create scheduler".to_string()))?;
        Ok(Scheduler {
            sched,
            pg: Arc::new(pg),
        })
    }

    pub async fn init_from_db(self) -> Result<Self, Error> {
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
                let cron_syntax = format!("0 */{} * * * *", task.interval_minutes);
                log::debug!("Creating job with cron syntax: {}", cron_syntax);
                let job = tokio_cron_scheduler::Job::new_async(
                    cron_syntax.as_str(),
                    move |uuid, mut _l| {
                        Box::pin(async move {
                            execute_job(uuid).await;
                        })
                    },
                )
                .change_context_lazy(|| {
                    Error::Message(format!(
                        "Failed to create job with cron syntax: '{}'",
                        cron_syntax
                    ))
                })?;
                self.sched.add(job).await.change_context_lazy(|| {
                    Error::Message("Failed to add job to scheduler".to_string())
                })?;

                log::info!(
                    "Successfully added scheduled task '{}' with cron '{}'",
                    task.name,
                    cron_syntax
                );
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
            if let Some(plugin) = plugins.get::<str>(&plugin_name) {
                if let Err(e) = plugin.update(1).await {
                    log::error!("Plugin update failed for {}: {}", plugin_name, e)
                }
            }
        });
    }
    job_set.join_all().await;
    log::info!("Plugin syn finished elapsed {:?}", start.elapsed());
}
