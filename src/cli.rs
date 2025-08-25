use std::{path::PathBuf, sync::Arc};

use crate::{
    config::settings::{Config, LoadConfigResult, load_config},
    domain::{
        models::{
            admin_user::{AdminUserPassword, AdminUsername, CreateAdminUserRequest},
            vuln_information::CreateVulnInformation,
        },
        ports::VulnService,
        services::Service,
    },
    errors::Error,
    input::http::http_server::{self, make_acceptor_and_advertise_addr},
    output::{
        db::{admin_user::AdminUserDao, pg::Pg},
        plugins,
        scheduler::Scheduler,
        worker::Worker,
    },
    utils::{
        auth::jwt::JWT,
        num_cpus,
        password_hash::compute_password_hash,
        runtime::{Runtime, make_runtime},
        telemetry,
    },
};
use clap::ValueHint;
use error_stack::{Result, ResultExt};

#[derive(Clone)]
pub struct Ctx<S: VulnService + Send + Sync + 'static> {
    pub vuln_service: Arc<S>,
    pub config: Arc<Config>,
    pub sched: Arc<Scheduler>,
    pub jwt: Arc<JWT>,
}

#[derive(Debug, clap::Parser)]
pub struct CommandStart {
    #[clap(short, long, help = "Path to config file", value_hint = ValueHint::FilePath)]
    config_file: PathBuf,
}

impl CommandStart {
    pub fn run(self) -> Result<(), Error> {
        error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::None);
        let LoadConfigResult { config, warnings } = load_config(self.config_file)?;
        let telemetry_runtime = make_telemetry_runtime();
        let mut drop_guards =
            telemetry::init(&telemetry_runtime, "vulnfeed", config.telemetry.clone());
        drop_guards.push(Box::new(telemetry_runtime));
        for warning in warnings {
            log::warn!("{warning}");
        }
        log::info!("server is starting with config: {config:#?}");
        let server_runtime = make_vulnfeed_runtime();
        server_runtime.block_on(run_server(&server_runtime, config))
    }
}

async fn run_server(server_rt: &Runtime, config: Config) -> Result<(), Error> {
    let make_error = || Error::Message("failed to start server".to_string());
    let (shutdown_tx, shutdown_rx) = mea::shutdown::new_pair();
    let (acceptor, advertise_addr) = make_acceptor_and_advertise_addr(
        &config.server.listen_addr,
        config.server.advertise_addr.as_deref(),
    )
    .await
    .change_context_lazy(make_error)?;

    let jwt = JWT::new(&config.auth.jwt.secret);

    let db = Pg::new(&config).await.change_context_lazy(make_error)?;
    let (sender, receiver) = mea::mpsc::unbounded::<CreateVulnInformation>();
    plugins::init(sender).change_context_lazy(make_error)?;

    let mut worker = Worker::new(receiver, db.clone());
    server_rt.spawn(async move { worker.run().await });

    let sched = Scheduler::try_new(db.clone())
        .await
        .change_context_lazy(make_error)?;

    let sched = sched.init_from_db().await.change_context_lazy(make_error)?;
    let vuln_service = Service::new(db);
    let ctx = Ctx {
        vuln_service: Arc::new(vuln_service),
        config: Arc::new(config),
        sched: Arc::new(sched),
        jwt: Arc::new(jwt),
    };

    let server = http_server::start_server(ctx, server_rt, shutdown_rx, acceptor, advertise_addr)
        .await
        .change_context_lazy(|| {
            Error::Message("A fatal error has occurred in server process.".to_string())
        })?;

    ctrlc::set_handler(move || shutdown_tx.shutdown()).change_context_lazy(|| {
        Error::Message("failed to setup ctrl-c signal handle".to_string())
    })?;

    server.await_shutdown().await;
    Ok(())
}

fn make_vulnfeed_runtime() -> Runtime {
    let parallelism = num_cpus().get();
    make_runtime("vulnfeed_runtime", "vulnfeed_thread", parallelism)
}

fn make_telemetry_runtime() -> Runtime {
    make_runtime("telemetry_runtime", "telemetry_thread", 1)
}

fn make_init_data_runtime() -> Runtime {
    make_runtime("init_data_runtime", "init_data_thread", 1)
}

#[derive(Debug, clap::Parser)]
pub struct CreateSuperUser {
    #[clap(short, long, help = "Path to config file", value_hint = ValueHint::FilePath)]
    config_file: PathBuf,
    #[clap(short, long, help = "Password")]
    password: String,
}

impl CreateSuperUser {
    pub fn run(self) -> Result<(), Error> {
        error_stack::Report::set_color_mode(error_stack::fmt::ColorMode::None);
        let LoadConfigResult { config, warnings } = load_config(self.config_file)?;
        let telemetry_runtime = make_telemetry_runtime();
        let mut drop_guards =
            telemetry::init(&telemetry_runtime, "init-data", config.telemetry.clone());
        drop_guards.push(Box::new(telemetry_runtime));
        for warning in warnings {
            log::warn!("{warning}");
        }
        let init_data_runtime = make_init_data_runtime();
        init_data_runtime.block_on(run_create_super_user(config, self.password))
    }
}

async fn run_create_super_user(config: Config, password: String) -> Result<(), Error> {
    let make_error = || Error::Message("failed to create super user".to_string());
    let password_hash = compute_password_hash(&password).change_context_lazy(make_error)?;
    let db = Pg::new(&config).await.change_context_lazy(make_error)?;
    let mut tx = db.pool.begin().await.change_context_lazy(make_error)?;

    let req = CreateAdminUserRequest::new(
        AdminUsername::try_new("admin").change_context_lazy(make_error)?,
        AdminUserPassword::try_new(password_hash).change_context_lazy(make_error)?,
    );
    AdminUserDao::create_super_user(&mut tx, req).await?;
    tx.commit().await.change_context_lazy(make_error)?;
    log::info!("create super user success");
    Ok(())
}
