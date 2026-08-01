//! SLOPOS Vision daemon — local OCR and subject lifting service.
//!
//! The daemon owns the model runtime and an ephemeral, session-scoped asset
//! store. Clients communicate through one newline-delimited JSON request per
//! local Unix-stream connection. No network listener, cloud fallback, model
//! download, or client-controlled output path is accepted.

use image::{DynamicImage, ImageFormat, ImageReader, Rgba, RgbaImage};
use parking_lot::Mutex;
use slopos_vision::{
    engine::{VisionEngine, VisionEngineConfig},
    error::VisionError as EngineError,
    types::{OcrOptions, SegmentationOptions},
};
use slopos_vision_protocol::{
    AcceptedResponse, ArtifactDescriptor, ArtifactRole, AssetDataResponse, AssetId,
    AssetLookupRequest, CapabilityResponse, ErrorCode, ExtractTextResult, ImageMediaType,
    ImageMetadata, ImageSource, JobId, JobLookupRequest, JobResultResponse, JobStatus,
    JobStatusResponse, LiftSubjectResult, OcrLine, OcrWord, PixelRect, ProtocolEnvelope,
    SubmitJobRequest, ValidationError, VisionError, VisionJob, VisionOperation, VisionRequest,
    VisionResponse, VISION_PROTOCOL_VERSION,
};
use std::collections::HashMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Cursor, Write};
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use uuid::Uuid;

const DEFAULT_MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_MAX_JOBS: usize = 8;
const MAX_ID_BYTES: usize = 128;
const MAX_RETAINED_JOBS: usize = 1024;
const MAX_ARTIFACT_COUNT: usize = 1024;
const MAX_ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;
const MAX_METADATA_BYTES: u64 = 4 * 1024 * 1024;
const METADATA_FILE_NAME: &str = ".metadata.json";
const ARTIFACT_CACHE_FULL: &str = "artifact cache is full";
const ASSET_UNAVAILABLE: &str = "asset is unavailable";
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

#[derive(Clone, Debug)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub models_dir: PathBuf,
    pub artifact_dir: PathBuf,
    pub max_frame_bytes: usize,
    pub max_jobs: usize,
}

impl DaemonConfig {
    pub fn from_environment() -> Result<Self, String> {
        let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| "XDG_RUNTIME_DIR is required for slopos-visiond".to_string())?;
        let socket_path = env::var_os("SLOPOS_VISION_SOCKET")
            .map(PathBuf::from)
            .unwrap_or_else(|| runtime_dir.join("slopos-i/vision.sock"));
        let models_dir = env::var_os("SLOPOS_VISION_MODELS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(default_models_dir);
        let artifact_dir = env::var_os("SLOPOS_VISION_ARTIFACT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| default_cache_dir().join("vision/assets"));

        Ok(Self {
            socket_path,
            runtime_dir,
            models_dir,
            artifact_dir,
            max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
            max_jobs: DEFAULT_MAX_JOBS,
        })
    }

    #[cfg(test)]
    fn for_test(socket_path: PathBuf, models_dir: PathBuf, artifact_dir: PathBuf) -> Self {
        let runtime_dir = socket_path
            .parent()
            .expect("test socket has parent")
            .to_path_buf();
        Self {
            socket_path,
            runtime_dir,
            models_dir,
            artifact_dir,
            max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
            max_jobs: 4,
        }
    }
}

fn default_models_dir() -> PathBuf {
    if let Some(data_home) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(data_home).join("slopos-i/models/vision");
    }
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".local/share/slopos-i/models/vision");
    }
    PathBuf::from("models/vision")
}

fn default_cache_dir() -> PathBuf {
    if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(cache_home).join("slopos-i");
    }
    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".cache/slopos-i");
    }
    PathBuf::from(".slopos-i-cache")
}

pub fn validate_socket_path(socket_path: &Path, runtime_dir: &Path) -> Result<(), String> {
    if !socket_path.is_absolute() || !runtime_dir.is_absolute() {
        return Err("Vision socket and runtime directory must be absolute".to_string());
    }
    if socket_path.file_name().is_none()
        || socket_path.file_name() == Some(std::ffi::OsStr::new("."))
    {
        return Err("Vision socket must name a file".to_string());
    }
    if socket_path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err("Vision socket path must not contain . or .. components".to_string());
    }
    if !socket_path.starts_with(runtime_dir) {
        return Err("Vision socket must stay inside the session runtime directory".to_string());
    }
    Ok(())
}

#[derive(Clone)]
pub struct ArtifactStore {
    root: PathBuf,
    state: Arc<Mutex<ArtifactState>>,
}

struct ArtifactState {
    metadata: HashMap<String, slopos_vision_protocol::StoredImage>,
    total_bytes: u64,
}

impl ArtifactStore {
    pub fn new(root: PathBuf) -> Result<Self, io::Error> {
        ensure_artifact_root(&root)?;
        cleanup_artifact_temps(&root)?;
        let metadata_path = root.join(METADATA_FILE_NAME);
        let metadata = load_artifact_metadata(&root, &metadata_path)?;
        let total_bytes = metadata
            .values()
            .map(|asset| asset.metadata.encoded_bytes)
            .try_fold(0_u64, |total, bytes| total.checked_add(bytes))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "asset cache size overflow")
            })?;
        if metadata.len() > MAX_ARTIFACT_COUNT || total_bytes > MAX_ARTIFACT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "asset cache exceeds its configured bound",
            ));
        }
        Ok(Self {
            root,
            state: Arc::new(Mutex::new(ArtifactState {
                metadata,
                total_bytes,
            })),
        })
    }

    pub fn path_for(&self, asset_id: &AssetId) -> Result<PathBuf, String> {
        asset_path_for(&self.root, asset_id)
    }

    fn put_png(
        &self,
        role: ArtifactRole,
        image: &RgbaImage,
    ) -> Result<ArtifactDescriptor, EngineError> {
        let asset_id = AssetId(format!("asset-{}", Uuid::new_v4().simple()));
        let path = self.path_for(&asset_id).map_err(EngineError::Unsupported)?;
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image.clone())
            .write_to(&mut bytes, ImageFormat::Png)
            .map_err(|error| EngineError::Unsupported(format!("PNG encoding failed: {error}")))?;
        let bytes = bytes.into_inner();
        if bytes.len() as u64 > MAX_ARTIFACT_BYTES {
            return Err(EngineError::EncodedImageTooLarge {
                max_bytes: MAX_ARTIFACT_BYTES,
                actual_bytes: bytes.len() as u64,
            });
        }
        let stored = slopos_vision_protocol::StoredImage {
            asset_id: asset_id.clone(),
            metadata: ImageMetadata {
                media_type: ImageMediaType::Png,
                encoded_bytes: bytes.len() as u64,
                dimensions: slopos_vision_protocol::PixelSize {
                    width: image.width(),
                    height: image.height(),
                },
                sha256: None,
                label: None,
            },
        };
        let mut state = self.state.lock();
        if state.metadata.len() >= MAX_ARTIFACT_COUNT
            || state
                .total_bytes
                .checked_add(bytes.len() as u64)
                .is_none_or(|total| total > MAX_ARTIFACT_BYTES)
        {
            return Err(EngineError::Unsupported(ARTIFACT_CACHE_FULL.to_string()));
        }
        write_atomic_file(&path, &bytes).map_err(EngineError::Io)?;
        state.total_bytes += bytes.len() as u64;
        state.metadata.insert(asset_id.0.clone(), stored.clone());
        if let Err(error) = persist_artifact_metadata(&self.root, &state.metadata) {
            state.metadata.remove(&asset_id.0);
            state.total_bytes -= bytes.len() as u64;
            let _ = fs::remove_file(&path);
            return Err(EngineError::Io(error));
        }
        Ok(ArtifactDescriptor {
            role,
            image: stored,
        })
    }

    fn get(&self, asset_id: &AssetId) -> Result<AssetDataResponse, EngineError> {
        let path = self
            .path_for(asset_id)
            .map_err(|_| EngineError::Unsupported("invalid asset id".to_string()))?;
        let stored = self
            .state
            .lock()
            .metadata
            .get(&asset_id.0)
            .cloned()
            .ok_or_else(|| EngineError::Unsupported(ASSET_UNAVAILABLE.to_string()))?;
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(EngineError::Unsupported(ASSET_UNAVAILABLE.to_string()));
            }
            Err(error) => return Err(EngineError::Io(error)),
        };
        if bytes.len() as u64 != stored.metadata.encoded_bytes {
            return Err(EngineError::Decode(
                "stored asset length changed".to_string(),
            ));
        }
        Ok(AssetDataResponse {
            asset: stored,
            bytes,
        })
    }
}

fn ensure_artifact_root(root: &Path) -> Result<(), io::Error> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "asset directory is not a real directory",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir_all(root)?,
        Err(error) => return Err(error),
    }
    let mut permissions = fs::symlink_metadata(root)?.permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(root, permissions)
}

fn asset_path_for(root: &Path, asset_id: &AssetId) -> Result<PathBuf, String> {
    asset_id
        .validate()
        .map_err(|_| "invalid asset id".to_string())?;
    let name = &asset_id.0;
    if name.len() > MAX_ID_BYTES
        || name == "."
        || name == ".."
        || Path::new(name).file_name().and_then(|value| value.to_str()) != Some(name)
        || name.chars().any(|ch| ch.is_control() || ch == '\0')
    {
        return Err("invalid asset path component".to_string());
    }
    Ok(root.join(name).with_extension("png"))
}

fn cleanup_artifact_temps(root: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let is_owned_temp = (name.starts_with("asset-") || name.starts_with(METADATA_FILE_NAME))
            && name.ends_with(".tmp");
        if is_owned_temp && metadata.file_type().is_file() {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn load_artifact_metadata(
    root: &Path,
    metadata_path: &Path,
) -> Result<HashMap<String, slopos_vision_protocol::StoredImage>, io::Error> {
    let bytes = match fs::symlink_metadata(metadata_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "asset metadata is not a regular file",
            ));
        }
        Ok(metadata) if metadata.len() > MAX_METADATA_BYTES => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "asset metadata exceeds its configured bound",
            ));
        }
        Ok(_) => fs::read(metadata_path)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(error) => return Err(error),
    };
    let metadata =
        serde_json::from_slice::<HashMap<String, slopos_vision_protocol::StoredImage>>(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "asset metadata is invalid"))?;
    if metadata.len() > MAX_ARTIFACT_COUNT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "asset metadata contains too many assets",
        ));
    }

    let metadata_len = metadata.len();
    let mut valid = HashMap::with_capacity(metadata.len());
    for (key, asset) in metadata {
        if key != asset.asset_id.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "asset metadata identifier mismatch",
            ));
        }
        let path = asset_path_for(root, &asset.asset_id)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "asset metadata is unsafe"))?;
        if asset.metadata.encoded_bytes > MAX_ARTIFACT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "asset metadata exceeds the cache bound",
            ));
        }
        let file_metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if file_metadata.file_type().is_symlink()
            || !file_metadata.is_file()
            || file_metadata.len() != asset.metadata.encoded_bytes
        {
            continue;
        }
        valid.insert(key, asset);
    }
    if valid.len() != metadata_len {
        persist_artifact_metadata(root, &valid)?;
    }
    Ok(valid)
}

fn persist_artifact_metadata(
    root: &Path,
    metadata: &HashMap<String, slopos_vision_protocol::StoredImage>,
) -> Result<(), io::Error> {
    let bytes = serde_json::to_vec(metadata).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "asset metadata is not serializable",
        )
    })?;
    if bytes.len() as u64 > MAX_METADATA_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "asset metadata exceeds its configured bound",
        ));
    }
    write_atomic_file(&root.join(METADATA_FILE_NAME), &bytes)
}

fn write_atomic_file(path: &Path, bytes: &[u8]) -> Result<(), io::Error> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("asset");
    let temporary = path.with_file_name(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        Uuid::new_v4().simple()
    ));
    let result = (|| {
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let mut permissions = output.metadata()?.permissions();
        permissions.set_mode(0o600);
        output.set_permissions(permissions)?;
        output.write_all(bytes)?;
        output.sync_all()?;
        drop(output);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

struct JobRecord {
    operation: VisionOperation,
    status: JobStatus,
    result: Option<slopos_vision_protocol::VisionResult>,
    error: Option<VisionError>,
    cancel: Arc<AtomicBool>,
}

fn mark_cancelled(job: &mut JobRecord) {
    job.cancel.store(true, Ordering::Release);
    job.status = JobStatus::Cancelled;
    job.error = Some(VisionError {
        code: ErrorCode::Cancelled,
        message: "Vision job was cancelled".to_string(),
        operation: Some(job.operation),
        retryable: false,
    });
}

fn prune_job_history(jobs: &mut HashMap<String, JobRecord>) {
    while jobs.len() >= MAX_RETAINED_JOBS {
        let Some(job_id) = jobs
            .iter()
            .find(|(_, job)| {
                matches!(
                    job.status,
                    JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled
                )
            })
            .map(|(job_id, _)| job_id.clone())
        else {
            break;
        };
        jobs.remove(&job_id);
    }
}

struct DaemonInner {
    config: DaemonConfig,
    engine: Arc<VisionEngine>,
    artifacts: ArtifactStore,
    jobs: Mutex<HashMap<String, JobRecord>>,
    job_tasks: Mutex<Vec<JoinHandle<()>>>,
    next_job: AtomicU64,
    shutting_down: AtomicBool,
    shutdown: watch::Sender<bool>,
}

#[derive(Clone)]
pub struct Daemon {
    inner: Arc<DaemonInner>,
}

impl Daemon {
    pub fn new(config: DaemonConfig) -> Result<Self, String> {
        validate_socket_path(&config.socket_path, &config.runtime_dir)?;
        fs::create_dir_all(&config.runtime_dir)
            .map_err(|_| "cannot create Vision runtime directory".to_string())?;
        if config.max_frame_bytes == 0 || config.max_frame_bytes > DEFAULT_MAX_FRAME_BYTES {
            return Err("Vision frame bound is outside the supported range".to_string());
        }
        if config.max_jobs == 0 {
            return Err("Vision job bound must be non-zero".to_string());
        }
        let engine = VisionEngine::load(VisionEngineConfig {
            models_dir: config.models_dir.clone(),
            ..Default::default()
        })
        .map_err(|error| {
            format!(
                "cannot load Vision model manifest: {}",
                map_vision_error(&error, None).message
            )
        })?;
        let artifacts = ArtifactStore::new(config.artifact_dir.clone())
            .map_err(|_| "cannot create Vision artifact store".to_string())?;
        let (shutdown, _) = watch::channel(false);
        Ok(Self {
            inner: Arc::new(DaemonInner {
                config,
                engine: Arc::new(engine),
                artifacts,
                jobs: Mutex::new(HashMap::new()),
                job_tasks: Mutex::new(Vec::new()),
                next_job: AtomicU64::new(1),
                shutting_down: AtomicBool::new(false),
                shutdown,
            }),
        })
    }

    pub async fn shutdown(&self) {
        if self.inner.shutting_down.swap(true, Ordering::AcqRel) {
            return;
        }
        self.inner.shutdown.send_replace(true);
        {
            let mut jobs = self.inner.jobs.lock();
            for job in jobs.values_mut() {
                if matches!(job.status, JobStatus::Queued | JobStatus::Running) {
                    mark_cancelled(job);
                }
            }
        }
        let tasks = std::mem::take(&mut *self.inner.job_tasks.lock());
        let _ = tokio::time::timeout(SHUTDOWN_GRACE, async move {
            for task in tasks {
                let _ = task.await;
            }
        })
        .await;
    }

    pub async fn handle_request(
        &self,
        envelope: ProtocolEnvelope<VisionRequest>,
    ) -> ProtocolEnvelope<VisionResponse> {
        if envelope.protocol_version != VISION_PROTOCOL_VERSION {
            return ProtocolEnvelope::new(VisionResponse::Error(VisionError {
                code: ErrorCode::UnsupportedProtocolVersion,
                message: "unsupported Vision protocol version".to_string(),
                operation: None,
                retryable: false,
            }));
        }

        let response = match envelope.payload {
            VisionRequest::Probe(_) => VisionResponse::Capabilities(CapabilityResponse {
                execution_mode: slopos_vision_protocol::ExecutionMode::LocalOnly,
                model_provisioning:
                    slopos_vision_protocol::ModelProvisioning::ImportedModelPackOnly,
                supported_operations: vec![
                    VisionOperation::ExtractText,
                    VisionOperation::LiftSubject,
                ],
            }),
            VisionRequest::SubmitJob(request) => self.submit(request),
            VisionRequest::GetJobStatus(request) => self.get_status(request),
            VisionRequest::GetJobResult(request) => self.get_result(request),
            VisionRequest::CancelJob(request) => self.cancel(request),
            VisionRequest::GetAsset(request) => self.get_asset(request),
        };
        ProtocolEnvelope::new(response)
    }

    fn submit(&self, request: SubmitJobRequest) -> VisionResponse {
        if self.inner.shutting_down.load(Ordering::Acquire) {
            return error_response(VisionError {
                code: ErrorCode::Internal,
                message: "Vision daemon is shutting down".to_string(),
                operation: None,
                retryable: true,
            });
        }
        if let Err(error) = request.validate(
            self.inner.engine.config().max_encoded_input_bytes,
            self.inner.engine.config().max_source_pixels,
        ) {
            return error_response(map_validation_error(error, None));
        }
        let operation = request.job.operation();
        self.reap_finished_job_tasks();
        let mut jobs = self.inner.jobs.lock();
        prune_job_history(&mut jobs);
        if jobs.len() >= MAX_RETAINED_JOBS {
            return error_response(VisionError {
                code: ErrorCode::Internal,
                message: "Vision job history is full".to_string(),
                operation: Some(operation),
                retryable: true,
            });
        }
        let active_jobs = jobs
            .values()
            .filter(|job| matches!(job.status, JobStatus::Queued | JobStatus::Running))
            .count();
        if active_jobs >= self.inner.config.max_jobs {
            return error_response(VisionError {
                code: ErrorCode::Internal,
                message: "Vision job queue is full".to_string(),
                operation: Some(operation),
                retryable: true,
            });
        }

        let job_id = JobId(format!(
            "job-{}",
            self.inner.next_job.fetch_add(1, Ordering::Relaxed)
        ));
        let cancel = Arc::new(AtomicBool::new(false));
        jobs.insert(
            job_id.0.clone(),
            JobRecord {
                operation,
                status: JobStatus::Queued,
                result: None,
                error: None,
                cancel: cancel.clone(),
            },
        );
        drop(jobs);

        let daemon = self.clone();
        let queued_job_id = job_id.clone();
        let task = tokio::spawn(async move {
            daemon.set_status(&queued_job_id, JobStatus::Running);
            let worker_daemon = daemon.clone();
            let worker_job_id = queued_job_id.clone();
            let job_result = tokio::task::spawn_blocking(move || {
                worker_daemon.execute_job(worker_job_id, request.job, cancel)
            })
            .await;
            match job_result {
                Ok((result, error)) => daemon.finish_job(&queued_job_id, result, error),
                Err(_) => daemon.finish_job(
                    &queued_job_id,
                    None,
                    Some(VisionError {
                        code: ErrorCode::Internal,
                        message: "Vision worker failed".to_string(),
                        operation: Some(operation),
                        retryable: true,
                    }),
                ),
            }
        });
        self.inner.job_tasks.lock().push(task);

        VisionResponse::Accepted(AcceptedResponse {
            job_id,
            operation,
            status: JobStatus::Queued,
        })
    }

    fn reap_finished_job_tasks(&self) {
        let mut tasks = self.inner.job_tasks.lock();
        let mut index = 0;
        while index < tasks.len() {
            if tasks[index].is_finished() {
                tasks.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn set_status(&self, job_id: &JobId, status: JobStatus) {
        if let Some(job) = self.inner.jobs.lock().get_mut(&job_id.0) {
            if job.status != JobStatus::Cancelled {
                job.status = status;
            }
        }
    }

    fn finish_job(
        &self,
        job_id: &JobId,
        result: Option<slopos_vision_protocol::VisionResult>,
        error: Option<VisionError>,
    ) {
        if let Some(job) = self.inner.jobs.lock().get_mut(&job_id.0) {
            if job.status == JobStatus::Cancelled {
                return;
            }
            job.status = if error.is_some() {
                JobStatus::Failed
            } else {
                JobStatus::Succeeded
            };
            job.result = result;
            job.error = error;
        }
    }

    fn execute_job(
        &self,
        _job_id: JobId,
        job: VisionJob,
        cancel: Arc<AtomicBool>,
    ) -> (
        Option<slopos_vision_protocol::VisionResult>,
        Option<VisionError>,
    ) {
        let operation = job.operation();
        if cancel.load(Ordering::Acquire) {
            return (
                None,
                Some(VisionError {
                    code: ErrorCode::Cancelled,
                    message: "Vision job was cancelled".to_string(),
                    operation: Some(operation),
                    retryable: false,
                }),
            );
        }
        let result = match job {
            VisionJob::ExtractText(request) => self.execute_ocr(request, cancel.clone()),
            VisionJob::LiftSubject(request) => self.execute_lift(request, cancel.clone()),
        };
        match result {
            Ok(result) => {
                if cancel.load(Ordering::Acquire) {
                    (
                        None,
                        Some(VisionError {
                            code: ErrorCode::Cancelled,
                            message: "Vision job was cancelled".to_string(),
                            operation: Some(operation),
                            retryable: false,
                        }),
                    )
                } else {
                    (Some(result), None)
                }
            }
            Err(error) => (None, Some(map_vision_error(&error, Some(operation)))),
        }
    }

    fn execute_ocr(
        &self,
        request: slopos_vision_protocol::ExtractTextJob,
        cancel: Arc<AtomicBool>,
    ) -> Result<slopos_vision_protocol::VisionResult, EngineError> {
        let image = self.source_image(&request.source)?;
        let ocr = self.inner.engine.extract_text(
            &image,
            OcrOptions {
                min_confidence: request.options.min_confidence.unwrap_or(0.5),
                cancel: Some(cancel),
            },
        )?;
        let lines = ocr
            .lines
            .into_iter()
            .map(|line| OcrLine {
                text: line.text,
                bounds: PixelRect {
                    x: line.bounds.x,
                    y: line.bounds.y,
                    width: line.bounds.width,
                    height: line.bounds.height,
                },
                confidence_milli: if request.options.include_line_confidence {
                    line.confidence.map(confidence_milli)
                } else {
                    None
                },
                words: if request.options.include_words {
                    line.words
                        .into_iter()
                        .map(|word| OcrWord {
                            text: word.text,
                            bounds: PixelRect {
                                x: word.bounds.x,
                                y: word.bounds.y,
                                width: word.bounds.width,
                                height: word.bounds.height,
                            },
                            confidence_milli: word.confidence.map(confidence_milli),
                        })
                        .collect()
                } else {
                    Vec::new()
                },
            })
            .collect::<Vec<_>>();
        let full_text = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(TheProtocolResult::ExtractText(ExtractTextResult {
            full_text,
            lines,
        }))
    }

    fn execute_lift(
        &self,
        request: slopos_vision_protocol::LiftSubjectJob,
        cancel: Arc<AtomicBool>,
    ) -> Result<slopos_vision_protocol::VisionResult, EngineError> {
        let image = self.source_image(&request.source)?;
        let lifted = self.inner.engine.lift_subject(
            &image,
            SegmentationOptions {
                cancel: Some(cancel),
                ..Default::default()
            },
        )?;
        let cutout = self
            .inner
            .artifacts
            .put_png(ArtifactRole::LiftedSubject, &lifted.image)?;
        let mask = if request.options.include_mask {
            let mask_image = RgbaImage::from_fn(lifted.mask.width, lifted.mask.height, |x, y| {
                let index = (y * lifted.mask.width + x) as usize;
                let alpha = lifted.mask.alpha.get(index).copied().unwrap_or(0);
                Rgba([255, 255, 255, alpha])
            });
            Some(
                self.inner
                    .artifacts
                    .put_png(ArtifactRole::SubjectMask, &mask_image)?,
            )
        } else {
            None
        };
        Ok(TheProtocolResult::LiftSubject(LiftSubjectResult {
            cutout,
            mask,
            opaque_pixel_count: Some(
                lifted.mask.alpha.iter().filter(|&&alpha| alpha > 0).count() as u64
            ),
        }))
    }

    fn source_image(&self, source: &ImageSource) -> Result<DynamicImage, EngineError> {
        let (metadata, bytes) = match source {
            ImageSource::Inline(image) => (&image.metadata, image.bytes.as_slice()),
            ImageSource::Stored(image) => {
                let data = self.inner.artifacts.get(&image.asset_id)?.bytes;
                return self.decode_and_validate(&image.metadata, &data);
            }
        };
        self.decode_and_validate(metadata, bytes)
    }

    fn decode_and_validate(
        &self,
        metadata: &ImageMetadata,
        bytes: &[u8],
    ) -> Result<DynamicImage, EngineError> {
        metadata
            .validate(
                self.inner.engine.config().max_encoded_input_bytes,
                self.inner.engine.config().max_source_pixels,
            )
            .map_err(|error| EngineError::Unsupported(format_validation_error(error)))?;
        if metadata.encoded_bytes != bytes.len() as u64 {
            return Err(EngineError::Decode(
                "encoded image length changed".to_string(),
            ));
        }
        let format = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|error| EngineError::Decode(error.to_string()))?
            .format()
            .ok_or_else(|| EngineError::Unsupported("unsupported image format".to_string()))?;
        if !media_type_matches(metadata.media_type, format) {
            return Err(EngineError::Unsupported(
                "image media type does not match encoded data".to_string(),
            ));
        }
        let image = self.inner.engine.decode_image_bytes(bytes)?;
        if image.width() != metadata.dimensions.width
            || image.height() != metadata.dimensions.height
        {
            return Err(EngineError::Decode(
                "decoded dimensions do not match request metadata".to_string(),
            ));
        }
        Ok(image)
    }

    fn get_status(&self, request: JobLookupRequest) -> VisionResponse {
        if let Err(error) = request.validate() {
            return error_response(map_validation_error(error, None));
        }
        let jobs = self.inner.jobs.lock();
        let Some(job) = jobs.get(&request.job_id.0) else {
            return error_response(missing_job_error());
        };
        VisionResponse::JobStatus(JobStatusResponse {
            job_id: request.job_id,
            operation: job.operation,
            status: job.status,
            error: job.error.clone(),
        })
    }

    fn get_result(&self, request: JobLookupRequest) -> VisionResponse {
        if let Err(error) = request.validate() {
            return error_response(map_validation_error(error, None));
        }
        let jobs = self.inner.jobs.lock();
        let Some(job) = jobs.get(&request.job_id.0) else {
            return error_response(missing_job_error());
        };
        VisionResponse::JobResult(JobResultResponse {
            job_id: request.job_id,
            operation: job.operation,
            status: job.status,
            result: job.result.clone(),
            error: job.error.clone(),
        })
    }

    fn cancel(&self, request: JobLookupRequest) -> VisionResponse {
        if let Err(error) = request.validate() {
            return error_response(map_validation_error(error, None));
        }
        let mut jobs = self.inner.jobs.lock();
        let Some(job) = jobs.get_mut(&request.job_id.0) else {
            return error_response(missing_job_error());
        };
        if matches!(job.status, JobStatus::Queued | JobStatus::Running) {
            mark_cancelled(job);
        }
        VisionResponse::JobStatus(JobStatusResponse {
            job_id: request.job_id,
            operation: job.operation,
            status: job.status,
            error: job.error.clone(),
        })
    }

    fn get_asset(&self, request: AssetLookupRequest) -> VisionResponse {
        if let Err(error) = request.validate() {
            return error_response(map_validation_error(error, None));
        }
        match self.inner.artifacts.get(&request.asset_id) {
            Ok(asset) => VisionResponse::Asset(asset),
            Err(error) => error_response(map_vision_error(&error, None)),
        }
    }

    async fn handle_connection(&self, stream: UnixStream) -> Result<(), String> {
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        loop {
            let mut frame = Vec::new();
            let read = (&mut reader)
                .take((self.inner.config.max_frame_bytes + 1) as u64)
                .read_until(b'\n', &mut frame)
                .await
                .map_err(|error| format!("Vision request read failed: {error}"))?;
            if read == 0 {
                return Ok(());
            }
            if read > self.inner.config.max_frame_bytes || frame.last() != Some(&b'\n') {
                return Err(
                    "Vision request exceeded the frame bound or delimiter was missing".to_string(),
                );
            }
            frame.pop();
            if frame.last() == Some(&b'\r') {
                frame.pop();
            }
            let response = match serde_json::from_slice::<ProtocolEnvelope<VisionRequest>>(&frame) {
                Ok(request) => self.handle_request(request).await,
                Err(error) => ProtocolEnvelope::new(VisionResponse::Error(VisionError {
                    code: ErrorCode::InvalidRequest,
                    message: format!("invalid Vision request: {error}"),
                    operation: None,
                    retryable: false,
                })),
            };
            let mut output = serde_json::to_vec(&response)
                .map_err(|error| format!("Vision response serialization failed: {error}"))?;
            if output.len() > self.inner.config.max_frame_bytes {
                return Err("Vision response exceeded the frame bound".to_string());
            }
            output.push(b'\n');
            write_half
                .write_all(&output)
                .await
                .map_err(|error| format!("Vision response write failed: {error}"))?;
        }
    }

    pub async fn serve(&self) -> Result<(), String> {
        remove_exact_socket(&self.inner.config.socket_path)?;
        if let Some(parent) = self.inner.config.socket_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create socket parent: {error}"))?;
        }
        let listener = UnixListener::bind(&self.inner.config.socket_path)
            .map_err(|error| format!("cannot bind Vision socket: {error}"))?;
        let mut permissions = fs::metadata(&self.inner.config.socket_path)
            .map_err(|error| format!("cannot inspect Vision socket: {error}"))?
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&self.inner.config.socket_path, permissions)
            .map_err(|error| format!("cannot secure Vision socket: {error}"))?;
        let _cleanup = SocketCleanup(self.inner.config.socket_path.clone());
        loop {
            if self.inner.shutting_down.load(Ordering::Acquire) {
                return Ok(());
            }
            tokio::select! {
                accepted = listener.accept() => {
                    let (stream, _) = accepted.map_err(|error| format!("Vision accept failed: {error}"))?;
                    let daemon = self.clone();
                    tokio::spawn(async move {
                        if let Err(error) = daemon.handle_connection(stream).await {
                            log::debug!("Vision client connection closed: {error}");
                        }
                    });
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
            }
        }
    }
}

struct SocketCleanup(PathBuf);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = remove_exact_socket(&self.0);
    }
}

fn remove_exact_socket(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() => {
            fs::remove_file(path)
                .map_err(|error| format!("cannot remove Vision socket: {error}"))?;
        }
        Ok(_) => return Err("Vision socket path exists but is not a Unix socket".to_string()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("cannot inspect Vision socket path: {error}")),
    }
    Ok(())
}

fn confidence_milli(confidence: f32) -> u16 {
    (confidence.clamp(0.0, 1.0) * 1000.0).round() as u16
}

fn media_type_matches(media_type: ImageMediaType, format: ImageFormat) -> bool {
    matches!(
        (media_type, format),
        (ImageMediaType::Png, ImageFormat::Png)
            | (ImageMediaType::Jpeg, ImageFormat::Jpeg)
            | (ImageMediaType::Webp, ImageFormat::WebP)
            | (ImageMediaType::Bmp, ImageFormat::Bmp)
            | (ImageMediaType::Tiff, ImageFormat::Tiff)
    )
}

fn error_response(error: VisionError) -> VisionResponse {
    VisionResponse::Error(error)
}

fn missing_job_error() -> VisionError {
    VisionError {
        code: ErrorCode::MissingAsset,
        message: "the requested Vision job is unavailable".to_string(),
        operation: None,
        retryable: false,
    }
}

fn format_validation_error(error: ValidationError) -> String {
    match error {
        ValidationError::InvalidIdentifier(_) => "invalid identifier".to_string(),
        ValidationError::InvalidFileLabel(_) => "invalid file label".to_string(),
        ValidationError::InvalidSha256 => "invalid image hash".to_string(),
        ValidationError::ZeroDimensions => "image dimensions must be non-zero".to_string(),
        ValidationError::EncodedBytesMismatch { .. } => "encoded image length mismatch".to_string(),
        ValidationError::EncodedBytesExceeded { .. } => "encoded image is too large".to_string(),
        ValidationError::PixelLimitExceeded { .. } => "image pixel limit exceeded".to_string(),
        ValidationError::InvalidConfidence => {
            "confidence must be finite and between 0 and 1".to_string()
        }
    }
}

fn map_validation_error(error: ValidationError, operation: Option<VisionOperation>) -> VisionError {
    let code = match error {
        ValidationError::InvalidIdentifier(_) => ErrorCode::InvalidIdentifier,
        ValidationError::InvalidFileLabel(_) => ErrorCode::InvalidFileLabel,
        ValidationError::InvalidSha256 => ErrorCode::InvalidRequest,
        ValidationError::ZeroDimensions => ErrorCode::InvalidRequest,
        ValidationError::EncodedBytesMismatch { .. } => ErrorCode::EncodedBytesMismatch,
        ValidationError::EncodedBytesExceeded { .. } => ErrorCode::EncodedBytesExceeded,
        ValidationError::PixelLimitExceeded { .. } => ErrorCode::PixelLimitExceeded,
        ValidationError::InvalidConfidence => ErrorCode::InvalidRequest,
    };
    VisionError {
        code,
        message: format_validation_error(error),
        operation,
        retryable: false,
    }
}

pub fn map_vision_error(error: &EngineError, operation: Option<VisionOperation>) -> VisionError {
    let (code, message, retryable) = match error {
        EngineError::ModelNotFound(_)
        | EngineError::ModelLoad(_)
        | EngineError::ManifestEntry(_) => (
            ErrorCode::ModelUnavailable,
            "the requested vision model is unavailable",
            false,
        ),
        EngineError::HashMismatch { .. } => (
            ErrorCode::HashMismatch,
            "the installed vision model failed verification",
            false,
        ),
        EngineError::ManifestLoad { .. } => (
            ErrorCode::ModelUnavailable,
            "the vision model manifest is unavailable",
            false,
        ),
        EngineError::EncodedImageTooLarge { .. } => (
            ErrorCode::EncodedBytesExceeded,
            "encoded image is too large",
            false,
        ),
        EngineError::ImageTooLarge { .. } => (
            ErrorCode::PixelLimitExceeded,
            "image pixel limit exceeded",
            false,
        ),
        EngineError::UnsupportedFormat(_) | EngineError::Decode(_) => (
            ErrorCode::DecodeFailed,
            "the image could not be decoded",
            false,
        ),
        EngineError::NoSubject => (ErrorCode::NoSubject, "no subject was found", false),
        EngineError::Cancelled => (ErrorCode::Cancelled, "Vision job was cancelled", false),
        EngineError::Inference(_) | EngineError::InvalidOutput(_) => {
            (ErrorCode::InferenceFailed, "vision inference failed", true)
        }
        EngineError::Unsupported(_) => (
            ErrorCode::InvalidRequest,
            "the Vision request is unsupported",
            false,
        ),
        EngineError::Io(_) => (
            ErrorCode::Internal,
            "the local Vision service encountered an I/O error",
            true,
        ),
    };
    VisionError {
        code,
        message: message.to_string(),
        operation,
        retryable,
    }
}

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();
    let config = match parse_args() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("slopos-visiond: {error}");
            std::process::exit(2);
        }
    };
    let daemon = match Daemon::new(config) {
        Ok(daemon) => daemon,
        Err(error) => {
            eprintln!("slopos-visiond: {error}");
            std::process::exit(1);
        }
    };
    let serving = daemon.clone();
    let server = tokio::spawn(async move { serving.serve().await });
    tokio::select! {
        result = server => {
            if let Ok(Err(error)) = result {
                eprintln!("slopos-visiond: {error}");
                std::process::exit(1);
            }
        }
        signal = tokio::signal::ctrl_c() => {
            if let Err(error) = signal {
                eprintln!("slopos-visiond: Ctrl-C handler failed: {error}");
            }
            daemon.shutdown().await;
        }
    }
}

fn parse_args() -> Result<DaemonConfig, String> {
    let mut config = DaemonConfig::from_environment()?;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("usage: slopos-visiond [--socket PATH] [--models-dir PATH] [--artifact-dir PATH]");
                std::process::exit(0);
            }
            "--socket" => {
                config.socket_path = PathBuf::from(args.next().ok_or("--socket needs a path")?);
            }
            "--models-dir" => {
                config.models_dir = PathBuf::from(args.next().ok_or("--models-dir needs a path")?);
            }
            "--artifact-dir" => {
                config.artifact_dir =
                    PathBuf::from(args.next().ok_or("--artifact-dir needs a path")?);
            }
            value if value.starts_with("--socket=") => {
                config.socket_path = PathBuf::from(&value[9..]);
            }
            value if value.starts_with("--models-dir=") => {
                config.models_dir = PathBuf::from(&value[13..]);
            }
            value if value.starts_with("--artifact-dir=") => {
                config.artifact_dir = PathBuf::from(&value[15..]);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(config)
}

// Alias used only to keep the protocol result constructors visually distinct
// from the engine result type in the execution functions above.
use slopos_vision_protocol::VisionResult as TheProtocolResult;

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbaImage};
    use slopos_vision_protocol::{
        ClientRequestId, ExtractTextJob, ExtractTextOptions, ImageMetadata, InlineImage,
        JobResultResponse, PixelSize, ProbeRequest,
    };
    use std::io::Cursor;
    use tempfile::tempdir;

    #[test]
    fn socket_and_artifact_paths_reject_unsafe_components() {
        let runtime = tempdir().unwrap();
        let outside = tempdir().unwrap();
        assert!(validate_socket_path(Path::new("vision.sock"), runtime.path()).is_err());
        assert!(validate_socket_path(&outside.path().join("vision.sock"), runtime.path()).is_err());
        assert!(validate_socket_path(&runtime.path().join("vision.sock"), runtime.path()).is_ok());

        let store = ArtifactStore::new(runtime.path().join("assets")).unwrap();
        let traversal = AssetId("../escape".into());
        assert!(store.path_for(&traversal).is_err());
    }

    #[test]
    fn vision_errors_map_without_exposing_sensitive_paths() {
        let sensitive_path = "/private/user/photo-model.onnx";
        let mapped = map_vision_error(
            &EngineError::ModelNotFound(sensitive_path.into()),
            Some(VisionOperation::LiftSubject),
        );
        assert_eq!(mapped.code, ErrorCode::ModelUnavailable);
        assert_eq!(mapped.operation, Some(VisionOperation::LiftSubject));
        assert!(!mapped.message.contains(sensitive_path));
        assert_eq!(mapped.message, "the requested vision model is unavailable");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn in_process_request_flow_queues_and_finishes_without_models() {
        let root = tempdir().unwrap();
        let models_dir = root.path().join("models");
        std::fs::create_dir(&models_dir).unwrap();
        std::fs::write(models_dir.join("manifest.toml"), "models = []").unwrap();
        let config = DaemonConfig::for_test(
            root.path().join("runtime").join("vision.sock"),
            models_dir.clone(),
            root.path().join("artifacts"),
        );
        let daemon = Daemon::new(config).unwrap();

        let probe = daemon
            .handle_request(ProtocolEnvelope::new(VisionRequest::Probe(ProbeRequest)))
            .await;
        assert!(matches!(probe.payload, VisionResponse::Capabilities(_)));

        let bytes = one_pixel_png();
        let submit = daemon
            .handle_request(ProtocolEnvelope::new(VisionRequest::SubmitJob(
                SubmitJobRequest {
                    client_request_id: Some(ClientRequestId("in-process".into())),
                    job: VisionJob::ExtractText(ExtractTextJob {
                        source: ImageSource::Inline(InlineImage {
                            metadata: ImageMetadata {
                                media_type: ImageMediaType::Png,
                                encoded_bytes: bytes.len() as u64,
                                dimensions: PixelSize {
                                    width: 1,
                                    height: 1,
                                },
                                sha256: None,
                                label: None,
                            },
                            bytes,
                        }),
                        options: ExtractTextOptions::default(),
                    }),
                },
            )))
            .await;
        let job_id = match submit.payload {
            VisionResponse::Accepted(accepted) => {
                assert!(matches!(
                    accepted.status,
                    JobStatus::Queued | JobStatus::Running
                ));
                accepted.job_id
            }
            other => panic!("unexpected submit response: {other:?}"),
        };

        let mut terminal: Option<JobResultResponse> = None;
        for _ in 0..100 {
            let result = daemon
                .handle_request(ProtocolEnvelope::new(VisionRequest::GetJobResult(
                    JobLookupRequest {
                        job_id: job_id.clone(),
                    },
                )))
                .await;
            if let VisionResponse::JobResult(response) = result.payload {
                if matches!(
                    response.status,
                    JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled
                ) {
                    terminal = Some(response);
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        let terminal = terminal.expect("job should reach a terminal status");
        assert_eq!(terminal.status, JobStatus::Failed);
        let error = terminal.error.expect("failed job should include an error");
        assert_eq!(error.code, ErrorCode::ModelUnavailable);
        assert!(!error.message.contains(&models_dir.display().to_string()));
        daemon.shutdown().await;
    }

    fn one_pixel_png() -> Vec<u8> {
        let image = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }
}
