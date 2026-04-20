use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use regex::Regex;

use crate::capability::{Capability, CapabilityDeclaration, PermissionLevel};
use crate::error::{SandboxError, WasmSandboxError};
use crate::tool::ToolCallArgs;

/// Static limits applied to each isolated WebAssembly execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmSandboxLimits {
    pub fuel: u64,
    pub epoch_deadline_ticks: u64,
    pub epoch_interval: Duration,
    pub max_memory_bytes: usize,
    pub max_table_elements: usize,
    pub max_instances: usize,
    pub max_tables: usize,
    pub max_memories: usize,
    pub max_output_bytes: usize,
}

impl Default for WasmSandboxLimits {
    fn default() -> Self {
        Self {
            fuel: 10_000_000,
            epoch_deadline_ticks: 1,
            epoch_interval: Duration::from_millis(50),
            max_memory_bytes: 64 * 1024 * 1024,
            max_table_elements: 10_000,
            max_instances: 1,
            max_tables: 4,
            max_memories: 4,
            max_output_bytes: 1024 * 1024,
        }
    }
}

/// A single WASI preopened directory and the permissions exposed to guest code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiDirectoryAccess {
    pub host_path: PathBuf,
    pub guest_path: String,
    pub read: bool,
    pub write: bool,
}

impl WasiDirectoryAccess {
    pub fn read_only(host_path: impl Into<PathBuf>, guest_path: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            guest_path: guest_path.into(),
            read: true,
            write: false,
        }
    }

    pub fn read_write(host_path: impl Into<PathBuf>, guest_path: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            guest_path: guest_path.into(),
            read: true,
            write: true,
        }
    }

    fn validate(&self) -> Result<(), SandboxError> {
        if !self.host_path.exists() {
            return Err(WasmSandboxError::MissingPreopenedDir(self.host_path.clone()).into());
        }
        if self.guest_path.trim().is_empty() {
            return Err(WasmSandboxError::InvalidGuestPath(self.guest_path.clone()).into());
        }
        if !self.read && !self.write {
            return Err(WasmSandboxError::Setup(format!(
                "directory `{}` exposes no permissions",
                self.host_path.display()
            ))
            .into());
        }
        Ok(())
    }

    fn recursive_pattern(&self) -> String {
        self.host_path.join("**").to_string_lossy().to_string()
    }
}

/// Capability-style WASI permission plan used for both execution and Guardian checks.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WasiAccessPlan {
    pub directories: Vec<WasiDirectoryAccess>,
    pub network_enabled: bool,
    pub allowed_networks: Vec<String>,
    pub env_allow_list: Vec<String>,
}

impl WasiAccessPlan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_directory(mut self, access: WasiDirectoryAccess) -> Self {
        self.directories.push(access);
        self
    }

    pub fn allow_env(mut self, name: impl Into<String>) -> Self {
        self.env_allow_list.push(name.into());
        self
    }

    pub fn allow_all_network(mut self) -> Self {
        self.network_enabled = true;
        self.allowed_networks.clear();
        self
    }

    pub fn allow_network(mut self, pattern: impl Into<String>) -> Self {
        self.network_enabled = true;
        self.allowed_networks.push(pattern.into());
        self
    }

    pub fn requires_write(&self) -> bool {
        self.directories.iter().any(|directory| directory.write)
    }

    pub fn default_permission_level(&self) -> PermissionLevel {
        if self.requires_write() || self.network_enabled {
            PermissionLevel::Elevated
        } else {
            PermissionLevel::Standard
        }
    }

    pub fn validate(&self) -> Result<(), SandboxError> {
        for directory in &self.directories {
            directory.validate()?;
        }

        if !self.network_enabled && !self.allowed_networks.is_empty() {
            return Err(WasmSandboxError::Setup(
                "network allowlist requires network_enabled = true".to_string(),
            )
            .into());
        }

        let _ = compile_network_allowlist(&self.allowed_networks)?;
        Ok(())
    }

    pub fn capability_declaration(
        &self,
        module_path: impl AsRef<Path>,
        permission_level: Option<PermissionLevel>,
    ) -> CapabilityDeclaration {
        let module_path = module_path.as_ref();
        let mut capabilities = vec![Capability::ReadFile {
            pattern: module_path.to_string_lossy().to_string(),
        }];

        for directory in &self.directories {
            if directory.read {
                capabilities.push(Capability::ReadFile {
                    pattern: directory.recursive_pattern(),
                });
            }
            if directory.write {
                capabilities.push(Capability::WriteFile {
                    pattern: directory.recursive_pattern(),
                });
            }
        }

        if self.network_enabled {
            if self.allowed_networks.is_empty() {
                capabilities.push(Capability::NetworkAccess {
                    pattern: ".*".to_string(),
                });
            } else {
                capabilities.extend(
                    self.allowed_networks
                        .iter()
                        .cloned()
                        .map(|pattern| Capability::NetworkAccess { pattern }),
                );
            }
        }

        let mut description = format!("Execute WASM module `{}`", module_path.display());
        if !self.env_allow_list.is_empty() {
            description.push_str(&format!("; env: {}", self.env_allow_list.join(", ")));
        }
        if self.network_enabled {
            description.push_str("; network enabled");
        }

        CapabilityDeclaration::new(
            "wasm_execute",
            description,
            permission_level.unwrap_or_else(|| self.default_permission_level()),
            capabilities,
        )
    }
}

/// Runtime parameters for a single WASM invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExecutionRequest {
    pub module_path: Option<PathBuf>,
    pub entrypoint: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub stdin: Vec<u8>,
    pub access_plan: WasiAccessPlan,
}

impl Default for WasmExecutionRequest {
    fn default() -> Self {
        Self {
            module_path: None,
            entrypoint: "_start".to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
            stdin: Vec::new(),
            access_plan: WasiAccessPlan::default(),
        }
    }
}

impl WasmExecutionRequest {
    pub fn tool_call_args(&self) -> Result<ToolCallArgs, SandboxError> {
        let module_path = self
            .module_path
            .clone()
            .ok_or(WasmSandboxError::MissingModulePath)?;
        let mut extra = HashMap::new();
        extra.insert("entrypoint".to_string(), self.entrypoint.clone());
        extra.insert(
            "network_enabled".to_string(),
            self.access_plan.network_enabled.to_string(),
        );

        Ok(ToolCallArgs {
            command: None,
            target_path: Some(module_path.clone()),
            is_write: self.access_plan.requires_write(),
            extra,
            capability: Some(self.access_plan.capability_declaration(&module_path, None)),
            task_id: None,
        })
    }

    fn validate(&self) -> Result<(), SandboxError> {
        if self.entrypoint.trim().is_empty() {
            return Err(WasmSandboxError::MissingEntrypoint(self.entrypoint.clone()).into());
        }
        self.access_plan.validate()?;
        for key in self.env.keys() {
            if !self
                .access_plan
                .env_allow_list
                .iter()
                .any(|allowed| allowed == key)
            {
                return Err(WasmSandboxError::EnvNotAllowed(key.clone()).into());
            }
        }
        Ok(())
    }
}

/// Captured result of a WASM execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExecutionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub fuel_consumed: u64,
}

/// Loaded, validated WebAssembly module ready for isolated instantiation.
#[derive(Clone)]
pub struct WasmModule {
    size_bytes: usize,
    #[cfg(feature = "wasm")]
    compiled: Arc<wasmtime::Module>,
    #[cfg(not(feature = "wasm"))]
    _raw: Arc<[u8]>,
}

impl WasmModule {
    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }
}

/// High-level Wasmtime + WASI sandbox wrapper used by MoreCode.
#[derive(Clone)]
pub struct WasmSandbox {
    pub limits: WasmSandboxLimits,
    #[cfg(feature = "wasm")]
    engine: Arc<wasmtime::Engine>,
}

impl WasmSandbox {
    pub fn new(limits: WasmSandboxLimits) -> Result<Self, SandboxError> {
        validate_limits(&limits)?;

        #[cfg(feature = "wasm")]
        {
            Ok(Self {
                engine: Arc::new(build_engine()?),
                limits,
            })
        }

        #[cfg(not(feature = "wasm"))]
        {
            Ok(Self { limits })
        }
    }

    pub fn create_restricted_wasi(
        &self,
        allowed_dirs: &[PathBuf],
    ) -> Result<WasiAccessPlan, SandboxError> {
        let mut plan = WasiAccessPlan::new();
        for (index, path) in allowed_dirs.iter().enumerate() {
            let guest_path = guest_path_for_index(index, path);
            plan = plan.with_directory(WasiDirectoryAccess::read_only(path.clone(), guest_path));
        }
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate_wasm_module(&self, bytes: &[u8]) -> Result<(), SandboxError> {
        const WASM_MAGIC: &[u8; 4] = b"\0asm";
        if bytes.len() < 4 || &bytes[..4] != WASM_MAGIC {
            return Err(WasmSandboxError::Validation(
                "input does not start with the WebAssembly magic header".to_string(),
            )
            .into());
        }

        #[cfg(feature = "wasm")]
        {
            wasmtime::Module::validate(&self.engine, bytes)
                .map_err(|error| WasmSandboxError::Validation(error.to_string()))?;
        }

        Ok(())
    }

    pub fn load_module(&self, bytes: &[u8]) -> Result<WasmModule, SandboxError> {
        self.validate_wasm_module(bytes)?;

        #[cfg(feature = "wasm")]
        {
            let module = wasmtime::Module::new(&self.engine, bytes)
                .map_err(|error| WasmSandboxError::Load(error.to_string()))?;
            Ok(WasmModule {
                size_bytes: bytes.len(),
                compiled: Arc::new(module),
            })
        }

        #[cfg(not(feature = "wasm"))]
        {
            let _ = bytes;
            Err(WasmSandboxError::FeatureDisabled.into())
        }
    }

    pub async fn execute(
        &self,
        bytes: &[u8],
        request: WasmExecutionRequest,
    ) -> Result<WasmExecutionResult, SandboxError> {
        let module = self.load_module(bytes)?;
        self.execute_loaded(&module, request).await
    }

    pub async fn execute_file(
        &self,
        module_path: impl AsRef<Path>,
        mut request: WasmExecutionRequest,
    ) -> Result<WasmExecutionResult, SandboxError> {
        let module_path = module_path.as_ref().to_path_buf();
        request
            .module_path
            .get_or_insert_with(|| module_path.clone());
        let bytes = tokio::fs::read(&module_path).await?;
        self.execute(&bytes, request).await
    }

    pub async fn execute_loaded(
        &self,
        module: &WasmModule,
        request: WasmExecutionRequest,
    ) -> Result<WasmExecutionResult, SandboxError> {
        request.validate()?;

        #[cfg(feature = "wasm")]
        {
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
            let ticker = tokio::spawn(spawn_epoch_ticker(
                Arc::clone(&self.engine),
                self.limits.epoch_interval,
                shutdown_rx,
            ));

            let limits = self.limits.clone();
            let engine = Arc::clone(&self.engine);
            let compiled = Arc::clone(&module.compiled);
            let result = tokio::task::spawn_blocking(move || {
                execute_loaded_sync(engine, compiled, limits, request)
            })
            .await
            .map_err(|error| WasmSandboxError::Join(error.to_string()))?;

            let _ = shutdown_tx.send(());
            ticker
                .await
                .map_err(|error| WasmSandboxError::Join(error.to_string()))?;

            result
        }

        #[cfg(not(feature = "wasm"))]
        {
            let _ = module;
            let _ = request;
            Err(WasmSandboxError::FeatureDisabled.into())
        }
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new(WasmSandboxLimits::default()).expect("default wasm sandbox should be valid")
    }
}

fn validate_limits(limits: &WasmSandboxLimits) -> Result<(), SandboxError> {
    if limits.fuel == 0
        || limits.epoch_deadline_ticks == 0
        || limits.epoch_interval.is_zero()
        || limits.max_memory_bytes == 0
        || limits.max_table_elements == 0
        || limits.max_instances == 0
        || limits.max_tables == 0
        || limits.max_memories == 0
        || limits.max_output_bytes == 0
    {
        return Err(WasmSandboxError::InvalidLimits.into());
    }
    Ok(())
}

fn guest_path_for_index(index: usize, path: &Path) -> String {
    if index == 0 {
        return ".".to_string();
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("./{name}"))
        .unwrap_or_else(|| format!("./dir-{index}"))
}

fn compile_network_allowlist(patterns: &[String]) -> Result<Vec<Regex>, SandboxError> {
    patterns
        .iter()
        .map(|pattern| {
            Regex::new(&format!("^(?:{pattern})$")).map_err(|error| {
                WasmSandboxError::InvalidNetworkPattern {
                    pattern: pattern.clone(),
                    reason: error.to_string(),
                }
                .into()
            })
        })
        .collect()
}

#[cfg(feature = "wasm")]
fn build_engine() -> Result<wasmtime::Engine, SandboxError> {
    let mut config = wasmtime::Config::new();
    config.consume_fuel(true);
    config.epoch_interruption(true);
    config.wasm_simd(true);

    wasmtime::Engine::new(&config)
        .map_err(|error| WasmSandboxError::Setup(error.to_string()).into())
}

#[cfg(feature = "wasm")]
async fn spawn_epoch_ticker(
    engine: Arc<wasmtime::Engine>,
    epoch_interval: Duration,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let mut interval = tokio::time::interval(epoch_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    interval.tick().await;

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => break,
            _ = interval.tick() => engine.increment_epoch(),
        }
    }
}

#[cfg(feature = "wasm")]
fn execute_loaded_sync(
    engine: Arc<wasmtime::Engine>,
    module: Arc<wasmtime::Module>,
    limits: WasmSandboxLimits,
    request: WasmExecutionRequest,
) -> Result<WasmExecutionResult, SandboxError> {
    use wasmtime::{Linker, Store, StoreLimits, StoreLimitsBuilder};
    use wasmtime_wasi::{
        p1,
        p2::pipe::{MemoryInputPipe, MemoryOutputPipe},
        DirPerms, FilePerms, I32Exit, WasiCtxBuilder,
    };

    struct WasmStoreState {
        wasi: p1::WasiP1Ctx,
        store_limits: StoreLimits,
    }

    let stdout = MemoryOutputPipe::new(limits.max_output_bytes);
    let stderr = MemoryOutputPipe::new(limits.max_output_bytes);
    let mut builder = WasiCtxBuilder::new();
    builder.stdout(stdout.clone()).stderr(stderr.clone());

    if !request.stdin.is_empty() {
        builder.stdin(MemoryInputPipe::new(request.stdin.clone()));
    }
    if !request.args.is_empty() {
        builder.args(&request.args);
    }
    for (key, value) in &request.env {
        builder.env(key, value);
    }

    if request.access_plan.network_enabled {
        builder.allow_tcp(true).allow_udp(true);
        if request.access_plan.allowed_networks.is_empty() {
            builder.inherit_network();
        } else {
            let allowlist = Arc::new(compile_network_allowlist(
                &request.access_plan.allowed_networks,
            )?);
            builder.allow_ip_name_lookup(true);
            builder.socket_addr_check(move |addr, _| {
                let allow = allowlist
                    .iter()
                    .any(|regex| regex.is_match(&addr.to_string()));
                Box::pin(async move { allow })
            });
        }
    }

    for directory in &request.access_plan.directories {
        let dir_perms = if directory.write {
            DirPerms::READ | DirPerms::MUTATE
        } else {
            DirPerms::READ
        };
        let file_perms = if directory.write {
            FilePerms::READ | FilePerms::WRITE
        } else {
            FilePerms::READ
        };

        builder
            .preopened_dir(
                &directory.host_path,
                &directory.guest_path,
                dir_perms,
                file_perms,
            )
            .map_err(|error| WasmSandboxError::Setup(error.to_string()))?;
    }

    let store_limits = StoreLimitsBuilder::new()
        .memory_size(limits.max_memory_bytes)
        .table_elements(limits.max_table_elements)
        .instances(limits.max_instances)
        .tables(limits.max_tables)
        .memories(limits.max_memories)
        .trap_on_grow_failure(true)
        .build();

    let mut store = Store::new(
        &engine,
        WasmStoreState {
            wasi: builder.build_p1(),
            store_limits,
        },
    );
    store.limiter(|state| &mut state.store_limits);
    store
        .set_fuel(limits.fuel)
        .map_err(|error| WasmSandboxError::Setup(error.to_string()))?;
    store.set_epoch_deadline(limits.epoch_deadline_ticks);

    let mut linker = Linker::new(&engine);
    p1::add_to_linker_sync(&mut linker, |state: &mut WasmStoreState| &mut state.wasi)
        .map_err(|error| WasmSandboxError::Setup(error.to_string()))?;

    let instance = linker
        .instantiate(&mut store, module.as_ref())
        .map_err(|error| WasmSandboxError::Instantiate(error.to_string()))?;

    let function = instance
        .get_func(&mut store, &request.entrypoint)
        .ok_or_else(|| WasmSandboxError::MissingEntrypoint(request.entrypoint.clone()))?;
    let function = function
        .typed::<(), ()>(&store)
        .map_err(|error| WasmSandboxError::Execute {
            entrypoint: request.entrypoint.clone(),
            reason: error.to_string(),
        })?;

    let mut exit_code = 0;
    if let Err(error) = function.call(&mut store, ()) {
        if let Some(exit) = error.downcast_ref::<I32Exit>() {
            exit_code = exit.0;
        } else {
            return Err(WasmSandboxError::Execute {
                entrypoint: request.entrypoint,
                reason: error.to_string(),
            }
            .into());
        }
    }

    let fuel_consumed = limits
        .fuel
        .saturating_sub(store.get_fuel().unwrap_or_default());

    Ok(WasmExecutionResult {
        exit_code,
        stdout: stdout.contents().to_vec(),
        stderr: stderr.contents().to_vec(),
        fuel_consumed,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::{
        WasiAccessPlan, WasiDirectoryAccess, WasmExecutionRequest, WasmSandbox, WasmSandboxLimits,
    };

    #[test]
    fn wasm_sandbox_rejects_invalid_limits() {
        let result = WasmSandbox::new(WasmSandboxLimits {
            fuel: 0,
            ..WasmSandboxLimits::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn restricted_wasi_requires_existing_paths() {
        let sandbox = WasmSandbox::new(WasmSandboxLimits::default()).unwrap();
        let temp = tempdir().unwrap();
        assert!(sandbox
            .create_restricted_wasi(&[temp.path().to_path_buf()])
            .is_ok());
        assert!(sandbox
            .create_restricted_wasi(&[temp.path().join("missing")])
            .is_err());
    }

    #[test]
    fn access_plan_builds_expected_capabilities() {
        let temp = tempdir().unwrap();
        let module_path = temp.path().join("guest.wasm");
        let expected_module_pattern = module_path.to_string_lossy().to_string();
        let plan = WasiAccessPlan::new()
            .with_directory(WasiDirectoryAccess::read_only(temp.path(), "."))
            .allow_network(r"127\.0\.0\.1:\d+")
            .allow_env("SAFE_TOKEN");

        let declaration = plan.capability_declaration(&module_path, None);
        assert!(declaration
            .capabilities
            .iter()
            .any(|capability| matches!(capability, crate::Capability::ReadFile { pattern } if pattern == &expected_module_pattern)));
        assert!(declaration
            .capabilities
            .iter()
            .any(|capability| matches!(capability, crate::Capability::NetworkAccess { pattern } if pattern == r"127\.0\.0\.1:\d+")));
    }

    #[test]
    fn request_builds_guardian_tool_args() {
        let request = WasmExecutionRequest {
            module_path: Some(PathBuf::from("guest.wasm")),
            access_plan: WasiAccessPlan::new()
                .with_directory(WasiDirectoryAccess::read_write("workspace", ".")),
            ..WasmExecutionRequest::default()
        };

        let args = request.tool_call_args().unwrap();
        assert_eq!(args.target_path, Some(PathBuf::from("guest.wasm")));
        assert!(args.is_write);
        assert!(args.capability.is_some());
    }

    #[test]
    fn access_plan_rejects_invalid_network_regex() {
        let plan = WasiAccessPlan::new().allow_network("(");
        assert!(plan.validate().is_err());
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn wasm_module_validation_checks_real_binary() {
        let sandbox = WasmSandbox::default();
        let bytes = wat::parse_str("(module (func (export \"run\")))").unwrap();
        assert!(sandbox.validate_wasm_module(&bytes).is_ok());
        assert!(sandbox.validate_wasm_module(b"\0asm\0").is_err());
    }

    #[cfg(feature = "wasm")]
    #[tokio::test]
    async fn wasm_execution_runs_entrypoint() {
        let sandbox = WasmSandbox::default();
        let bytes = wat::parse_str("(module (func (export \"run\")))").unwrap();
        let result = sandbox
            .execute(
                &bytes,
                WasmExecutionRequest {
                    entrypoint: "run".to_string(),
                    ..WasmExecutionRequest::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }

    #[cfg(feature = "wasm")]
    #[tokio::test]
    async fn wasm_execution_interrupts_infinite_loop() {
        let sandbox = WasmSandbox::new(WasmSandboxLimits {
            fuel: 1_000_000,
            epoch_interval: std::time::Duration::from_millis(10),
            ..WasmSandboxLimits::default()
        })
        .unwrap();
        let bytes = wat::parse_str("(module (func (export \"spin\") (loop br 0)))").unwrap();

        let error = sandbox
            .execute(
                &bytes,
                WasmExecutionRequest {
                    entrypoint: "spin".to_string(),
                    ..WasmExecutionRequest::default()
                },
            )
            .await
            .unwrap_err();

        let message = error.to_string();
        assert!(
            message.contains("failed to execute wasm entrypoint")
                || message.contains("deadline")
                || message.contains("interrupt")
                || message.contains("fuel"),
            "{message}"
        );
    }
}
