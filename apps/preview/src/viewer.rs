use image::{DynamicImage, ImageFormat, ImageReader};
use slopos_kit::event::{Event, KeyCode};
use slopos_kit::panel::Panel;
use slopos_kit::scroll_view::ScrollView;
use slopos_kit::theme::ThemeContext;
use slopos_kit::widget::{Widget, WidgetState};
use slopos_kit::{AccessibilityNode, AccessibilityRole, Button, EventResult, Label};
use slopos_kit::{LayoutConstraint, Rect, Size};
use slopos_sdk::EventLoopWaker;
use slopos_vision_client::{VisionClient, VisionClientConfig};
use slopos_vision_protocol::{
    ArtifactRole, AssetDataResponse, ClientRequestId, ExtractTextJob, ExtractTextOptions,
    FileLabel, ImageMediaType, ImageMetadata, ImageSource, InlineImage, JobId, JobResultResponse,
    JobStatus, LiftSubjectJob, LiftSubjectOptions, PixelSize, SubmitJobRequest, VisionJob,
    VisionResult, MAX_FILE_STEM_LEN,
};
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_ENCODED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_PIXELS: u64 = 40_000_000;
const MAX_EXTRACTED_TEXT_BYTES: usize = 4 * 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_OUTPUT_COLLISION_ATTEMPTS: usize = 32;
const MAX_ARTIFACT_STEM_BYTES: usize = 96;
const THUMBNAIL_MAX_WIDTH: u32 = 96;
const THUMBNAIL_MAX_HEIGHT: u32 = 64;
const MIN_ZOOM: f32 = 0.01;
const MAX_ZOOM: f32 = 8.0;
const SCROLLBAR_WIDTH: f32 = 12.0;
const VISION_POLL_INTERVAL: Duration = Duration::from_millis(250);
// CPU-only OCR/segmentation can take several minutes on the supported UTM
// guest. Keep the watcher bounded, but allow a legitimate local job to reach
// its terminal event instead of abandoning it before the daemon completes.
const VISION_JOB_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const VISION_EVENT_QUEUE_CAPACITY: usize = 8;

static ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoomMode {
    Fit,
    Manual,
}

#[derive(Debug)]
struct LoadedImage {
    path: PathBuf,
    width: u32,
    height: u32,
    encoded: Vec<u8>,
    media_type: Option<ImageMediaType>,
    pixels: Vec<[f32; 4]>,
    pixel_width: u32,
    pixel_height: u32,
}

/// Parse the deliberately small Preview CLI: zero or one image path.
pub(crate) fn parse_cli_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Option<PathBuf>, String> {
    let mut args = args.into_iter();
    let _program = args.next();
    let path = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err("expected zero or one image path".to_string());
    }
    if path
        .as_deref()
        .is_some_and(|path| path.as_os_str().is_empty())
    {
        return Err("image path must not be empty".to_string());
    }
    Ok(path)
}

fn validate_image_path(path: &Path, max_encoded_bytes: u64) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("cannot inspect path: {error}"))?;
    if !metadata.is_file() {
        return Err("path is not a regular file".to_string());
    }
    if metadata.len() > max_encoded_bytes {
        return Err(format!(
            "encoded image is too large ({} bytes; limit is {} bytes)",
            metadata.len(),
            max_encoded_bytes
        ));
    }
    Ok(metadata.len())
}

fn media_type_for_format(format: ImageFormat) -> Option<ImageMediaType> {
    match format {
        ImageFormat::Png => Some(ImageMediaType::Png),
        ImageFormat::Jpeg => Some(ImageMediaType::Jpeg),
        ImageFormat::WebP => Some(ImageMediaType::Webp),
        ImageFormat::Bmp => Some(ImageMediaType::Bmp),
        ImageFormat::Tiff => Some(ImageMediaType::Tiff),
        _ => None,
    }
}

fn decode_encoded_image_limited(
    data: &[u8],
) -> Result<(DynamicImage, Option<ImageMediaType>), String> {
    if data.len() as u64 > MAX_ENCODED_BYTES {
        return Err("encoded image exceeds the Preview size limit".to_string());
    }

    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|error| format!("cannot identify image format: {error}"))?;
    let format = reader
        .format()
        .ok_or_else(|| "unsupported image format".to_string())?;

    let (width, height) = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|error| format!("cannot identify image dimensions: {error}"))?
        .into_dimensions()
        .map_err(|error| format!("cannot read image dimensions: {error}"))?;
    let pixels = (width as u64).saturating_mul(height as u64);
    if width == 0 || height == 0 {
        return Err("image has empty dimensions".to_string());
    }
    if pixels > MAX_SOURCE_PIXELS {
        return Err(format!(
            "image is too large ({} pixels; limit is {})",
            pixels, MAX_SOURCE_PIXELS
        ));
    }

    let image = reader
        .decode()
        .map_err(|error| format!("cannot decode image: {error}"))?;
    Ok((image, media_type_for_format(format)))
}

fn decode_image_limited(
    path: &Path,
) -> Result<(DynamicImage, Vec<u8>, Option<ImageMediaType>), String> {
    let encoded_bytes = validate_image_path(path, MAX_ENCODED_BYTES)?;
    let data = fs::read(path).map_err(|error| format!("cannot read image: {error}"))?;
    if data.len() as u64 > MAX_ENCODED_BYTES || data.len() as u64 != encoded_bytes {
        return Err("image changed while it was being read".to_string());
    }

    let (image, media_type) = decode_encoded_image_limited(&data)?;
    Ok((image, data, media_type))
}

fn load_image(path: &Path) -> Result<LoadedImage, String> {
    let (image, encoded, media_type) = decode_image_limited(path)?;
    let width = image.width();
    let height = image.height();
    let thumbnail = image.thumbnail(THUMBNAIL_MAX_WIDTH, THUMBNAIL_MAX_HEIGHT);
    let rgba = thumbnail.to_rgba8();
    let pixel_width = rgba.width();
    let pixel_height = rgba.height();
    let pixels = rgba
        .pixels()
        .enumerate()
        .map(|(index, pixel)| {
            let x = index as u32 % pixel_width;
            let y = index as u32 / pixel_width;
            let checker = if (x + y).is_multiple_of(2) {
                0.91
            } else {
                0.78
            };
            let alpha = f32::from(pixel[3]) / 255.0;
            [
                f32::from(pixel[0]) / 255.0 * alpha + checker * (1.0 - alpha),
                f32::from(pixel[1]) / 255.0 * alpha + checker * (1.0 - alpha),
                f32::from(pixel[2]) / 255.0 * alpha + checker * (1.0 - alpha),
                1.0,
            ]
        })
        .collect();

    Ok(LoadedImage {
        path: path.to_path_buf(),
        width,
        height,
        encoded,
        media_type,
        pixels,
        pixel_width,
        pixel_height,
    })
}

fn clamp_zoom(zoom: f32) -> f32 {
    if zoom.is_nan() {
        MIN_ZOOM
    } else if zoom.is_finite() {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    } else if zoom.is_sign_positive() {
        MAX_ZOOM
    } else {
        MIN_ZOOM
    }
}

fn fit_zoom(image_width: u32, image_height: u32, viewport_width: f32, viewport_height: f32) -> f32 {
    if image_width == 0 || image_height == 0 || viewport_width <= 0.0 || viewport_height <= 0.0 {
        return 1.0;
    }
    let width_ratio = viewport_width / image_width as f32;
    let height_ratio = viewport_height / image_height as f32;
    clamp_zoom(width_ratio.min(height_ratio))
}

fn solid_panel(fill: [f32; 4]) -> Panel {
    let mut panel = Panel::new();
    panel.themed = false;
    panel.fill = fill;
    panel.beveled = false;
    panel.raised = false;
    panel.bordered = false;
    panel
}

struct ImageCanvas {
    state: WidgetState,
    cells: Vec<Panel>,
    path: PathBuf,
    encoded: Vec<u8>,
    media_type: Option<ImageMediaType>,
    pixel_width: u32,
    pixel_height: u32,
    source_width: u32,
    source_height: u32,
    zoom: f32,
}

impl ImageCanvas {
    fn empty() -> Self {
        Self {
            state: WidgetState::new(),
            cells: Vec::new(),
            path: PathBuf::new(),
            encoded: Vec::new(),
            media_type: None,
            pixel_width: 0,
            pixel_height: 0,
            source_width: 0,
            source_height: 0,
            zoom: 1.0,
        }
    }

    fn from_loaded(image: LoadedImage) -> Self {
        let cells = image.pixels.into_iter().map(solid_panel).collect();
        Self {
            state: WidgetState::new(),
            cells,
            path: image.path,
            encoded: image.encoded,
            media_type: image.media_type,
            pixel_width: image.pixel_width,
            pixel_height: image.pixel_height,
            source_width: image.width,
            source_height: image.height,
            zoom: 1.0,
        }
    }

    fn natural_size(&self) -> Size {
        if self.source_width == 0 || self.source_height == 0 {
            return Size::new(1.0, 1.0);
        }
        Size::new(
            self.source_width as f32 * self.zoom,
            self.source_height as f32 * self.zoom,
        )
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = clamp_zoom(zoom);
        let size = self.natural_size();
        let rect = self.rect();
        self.state.rect = Rect::new(rect.x, rect.y, size.width, size.height);
        self.position_cells();
    }

    fn position_cells(&mut self) {
        if self.pixel_width == 0 || self.pixel_height == 0 {
            return;
        }
        let rect = self.rect();
        let cell_width = rect.width / self.pixel_width as f32;
        let cell_height = rect.height / self.pixel_height as f32;
        for (index, cell) in self.cells.iter_mut().enumerate() {
            let x = index as u32 % self.pixel_width;
            let y = index as u32 / self.pixel_width;
            let cell_rect = Rect::new(
                rect.x + x as f32 * cell_width,
                rect.y + y as f32 * cell_height,
                cell_width,
                cell_height,
            );
            cell.set_rect(cell_rect);
        }
    }
}

impl Widget for ImageCanvas {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn set_rect(&mut self, rect: Rect) {
        self.state.rect = rect;
        self.position_cells();
    }

    fn layout(&mut self, _constraint: LayoutConstraint) -> Size {
        let size = self.natural_size();
        let rect = self.rect();
        self.state.rect = Rect::new(rect.x, rect.y, size.width, size.height);
        self.position_cells();
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::Image, "Image"))
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.cells.iter().map(|cell| cell as &dyn Widget).collect()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        self.cells
            .iter_mut()
            .map(|cell| cell as &mut dyn Widget)
            .collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct PreviewView {
    state: WidgetState,
    source_path: Option<PathBuf>,
    source_dimensions: Option<(u32, u32)>,
    zoom: f32,
    zoom_mode: ZoomMode,
    background: Panel,
    toolbar_background: Panel,
    status_background: Panel,
    filename_label: Label,
    dimensions_label: Label,
    zoom_label: Label,
    empty_hint: Label,
    open_button: Button,
    zoom_out_button: Button,
    zoom_in_button: Button,
    fit_button: Button,
    actual_size_button: Button,
    extract_text_button: Button,
    lift_subject_button: Button,
    image_scroll: ScrollView,
    scrollbar: Panel,
    status_label: Label,
    status_text: String,
    vision_event_tx: mpsc::SyncSender<VisionJobEvent>,
    vision_event_rx: mpsc::Receiver<VisionJobEvent>,
    vision_event_waker: EventLoopWaker,
    vision_submission_generation: Arc<AtomicU64>,
    active_vision_submission: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum ViewAction {
    Open,
    ZoomIn,
    ZoomOut,
    Fit,
    ActualSize,
    ExtractText,
    LiftSubject,
}

#[derive(Debug, Clone, Copy)]
enum VisionAction {
    ExtractText,
    LiftSubject,
}

impl VisionAction {
    fn label(self) -> &'static str {
        match self {
            Self::ExtractText => "Extract Text",
            Self::LiftSubject => "Lift Subject",
        }
    }
}

#[derive(Debug)]
struct VisionJobTerminal {
    action: VisionAction,
    job_id: JobId,
    job: JobResultResponse,
    asset: Option<AssetDataResponse>,
    asset_error: Option<String>,
}

#[derive(Debug)]
enum VisionJobEvent {
    Status {
        submission_id: u64,
        action: VisionAction,
        status: JobStatus,
    },
    Terminal {
        submission_id: u64,
        result: Box<VisionJobTerminal>,
    },
    Timeout {
        submission_id: u64,
        action: VisionAction,
        elapsed: Duration,
        last_error: Option<String>,
    },
}

impl VisionJobEvent {
    fn submission_id(&self) -> u64 {
        match self {
            Self::Status { submission_id, .. }
            | Self::Terminal { submission_id, .. }
            | Self::Timeout { submission_id, .. } => *submission_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisionPollState {
    Pending,
    Terminal,
    TimedOut,
}

fn vision_poll_state(
    status: JobStatus,
    started_at: Instant,
    now: Instant,
    timeout: Duration,
) -> VisionPollState {
    if matches!(
        status,
        JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled | JobStatus::Rejected
    ) {
        VisionPollState::Terminal
    } else if now.saturating_duration_since(started_at) >= timeout {
        VisionPollState::TimedOut
    } else {
        VisionPollState::Pending
    }
}

struct VisionJobWatch {
    submission_id: u64,
    action: VisionAction,
    job_id: JobId,
}

fn vision_submission_is_current(generation: &AtomicU64, submission_id: u64) -> bool {
    generation.load(Ordering::Acquire) == submission_id
}

fn send_vision_timeout(
    event_tx: &mpsc::SyncSender<VisionJobEvent>,
    event_waker: &EventLoopWaker,
    generation: &AtomicU64,
    watch: &VisionJobWatch,
    started_at: Instant,
    last_error: Option<String>,
) {
    if !vision_submission_is_current(generation, watch.submission_id) {
        return;
    }
    if event_tx
        .send(VisionJobEvent::Timeout {
            submission_id: watch.submission_id,
            action: watch.action,
            elapsed: started_at.elapsed().min(VISION_JOB_TIMEOUT),
            last_error,
        })
        .is_ok()
    {
        event_waker.wake();
    }
}

fn sleep_until_next_vision_poll(started_at: Instant) {
    let elapsed = started_at.elapsed();
    if elapsed < VISION_JOB_TIMEOUT {
        thread::sleep(VISION_POLL_INTERVAL.min(VISION_JOB_TIMEOUT - elapsed));
    }
}

fn retrieve_lifted_subject_asset(
    client: &VisionClient,
    job: &JobResultResponse,
) -> (Option<AssetDataResponse>, Option<String>) {
    let Some(VisionResult::LiftSubject(result)) = job.result.as_ref() else {
        return (None, None);
    };
    if result.cutout.role != ArtifactRole::LiftedSubject {
        return (None, None);
    }
    match client.get_asset(result.cutout.image.asset_id.clone()) {
        Ok(asset) => (Some(asset), None),
        Err(error) => (None, Some(error.to_string())),
    }
}

fn watch_vision_job(
    client: VisionClient,
    event_tx: mpsc::SyncSender<VisionJobEvent>,
    event_waker: EventLoopWaker,
    generation: Arc<AtomicU64>,
    watch: VisionJobWatch,
) {
    let started_at = Instant::now();
    let mut last_status = None;
    let mut last_error = None;

    loop {
        if !vision_submission_is_current(&generation, watch.submission_id) {
            return;
        }
        if started_at.elapsed() >= VISION_JOB_TIMEOUT {
            send_vision_timeout(
                &event_tx,
                &event_waker,
                &generation,
                &watch,
                started_at,
                last_error,
            );
            return;
        }

        match client.get_result(watch.job_id.clone()) {
            Ok(job) => {
                last_error = None;
                match vision_poll_state(job.status, started_at, Instant::now(), VISION_JOB_TIMEOUT)
                {
                    VisionPollState::Pending => {
                        if last_status != Some(job.status) {
                            last_status = Some(job.status);
                            if event_tx
                                .try_send(VisionJobEvent::Status {
                                    submission_id: watch.submission_id,
                                    action: watch.action,
                                    status: job.status,
                                })
                                .is_ok()
                            {
                                event_waker.wake();
                            }
                        }
                        sleep_until_next_vision_poll(started_at);
                    }
                    VisionPollState::Terminal => {
                        let (asset, asset_error) = if job.status == JobStatus::Succeeded {
                            retrieve_lifted_subject_asset(&client, &job)
                        } else {
                            (None, None)
                        };
                        if !vision_submission_is_current(&generation, watch.submission_id) {
                            return;
                        }
                        if event_tx
                            .send(VisionJobEvent::Terminal {
                                submission_id: watch.submission_id,
                                result: Box::new(VisionJobTerminal {
                                    action: watch.action,
                                    job_id: watch.job_id.clone(),
                                    job,
                                    asset,
                                    asset_error,
                                }),
                            })
                            .is_ok()
                        {
                            event_waker.wake();
                        }
                        return;
                    }
                    VisionPollState::TimedOut => {
                        send_vision_timeout(
                            &event_tx,
                            &event_waker,
                            &generation,
                            &watch,
                            started_at,
                            None,
                        );
                        return;
                    }
                }
            }
            Err(error) => {
                last_error = Some(error.to_string());
                sleep_until_next_vision_poll(started_at);
            }
        }
    }
}

fn spawn_vision_job_watcher(
    client: VisionClient,
    event_tx: mpsc::SyncSender<VisionJobEvent>,
    event_waker: EventLoopWaker,
    generation: Arc<AtomicU64>,
    watch: VisionJobWatch,
) -> Result<(), String> {
    thread::Builder::new()
        .name("preview-vision-watch".to_string())
        .spawn(move || watch_vision_job(client, event_tx, event_waker, generation, watch))
        .map(|_| ())
        .map_err(|error| format!("cannot start Vision job watcher: {error}"))
}

impl PreviewView {
    pub fn new(path: Option<PathBuf>) -> Self {
        let filename = path
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "No image selected".to_string());
        let (loaded, status) = match path.as_deref() {
            Some(path) => match load_image(path) {
                Ok(image) => (
                    Some(image),
                    "Ready — image decoded; Vision uses the local daemon when available."
                        .to_string(),
                ),
                Err(error) => (None, format!("Cannot open {filename}: {error}")),
            },
            None => (
                None,
                "No image selected — launch Preview with an image path or use File > Open..."
                    .to_string(),
            ),
        };
        let source_dimensions = loaded.as_ref().map(|image| (image.width, image.height));
        let canvas = loaded
            .map(ImageCanvas::from_loaded)
            .unwrap_or_else(ImageCanvas::empty);
        let mut image_scroll = ScrollView::new();
        image_scroll.scrollable_x = true;
        image_scroll.scrollable_y = true;
        image_scroll.set_content(Box::new(canvas));
        let (vision_event_tx, vision_event_rx) = mpsc::sync_channel(VISION_EVENT_QUEUE_CAPACITY);

        let mut view = Self {
            state: WidgetState::new(),
            source_path: path,
            source_dimensions,
            zoom: 1.0,
            zoom_mode: ZoomMode::Fit,
            background: solid_panel([0.93, 0.93, 0.92, 1.0]),
            toolbar_background: solid_panel([0.86, 0.86, 0.85, 1.0]),
            status_background: solid_panel([0.86, 0.86, 0.85, 1.0]),
            filename_label: Label::new(filename),
            dimensions_label: Label::new("No image loaded"),
            zoom_label: Label::new("No image"),
            empty_hint: Label::new("No image selected"),
            open_button: Button::new("Open..."),
            zoom_out_button: Button::new("-"),
            zoom_in_button: Button::new("+"),
            fit_button: Button::new("Fit"),
            actual_size_button: Button::new("100%"),
            extract_text_button: Button::new("Extract Text"),
            lift_subject_button: Button::new("Lift Subject"),
            image_scroll,
            scrollbar: solid_panel([0.50, 0.50, 0.48, 1.0]),
            status_label: Label::new(status.clone()),
            status_text: status,
            vision_event_tx,
            vision_event_rx,
            vision_event_waker: EventLoopWaker::default(),
            vision_submission_generation: Arc::new(AtomicU64::new(0)),
            active_vision_submission: None,
        };
        view.update_labels();
        view
    }

    /// Install the SDK wake handle after the application has been created.
    /// Vision watcher threads use it to wake the event loop when a result is
    /// ready, while the default handle keeps isolated widget tests inert.
    pub fn set_event_loop_waker(&mut self, waker: EventLoopWaker) {
        self.vision_event_waker = waker;
    }

    pub fn window_title(&self) -> String {
        match self
            .source_path
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
        {
            Some(filename) => format!("Preview - {filename}"),
            None => "Preview".to_string(),
        }
    }

    pub fn handle_action(&mut self, action: &str) {
        let action = match action {
            "com.slopos.preview.file.open" => Some(ViewAction::Open),
            "com.slopos.preview.zoom.in" => Some(ViewAction::ZoomIn),
            "com.slopos.preview.zoom.out" => Some(ViewAction::ZoomOut),
            "com.slopos.preview.zoom.fit" => Some(ViewAction::Fit),
            "com.slopos.preview.zoom.actual_size" => Some(ViewAction::ActualSize),
            "com.slopos.preview.vision.extract_text" => Some(ViewAction::ExtractText),
            "com.slopos.preview.vision.lift_subject" => Some(ViewAction::LiftSubject),
            other => match other.rsplit_once('.') {
                Some((_, "open")) => Some(ViewAction::Open),
                Some((_, "zoom_in")) => Some(ViewAction::ZoomIn),
                Some((_, "zoom_out")) => Some(ViewAction::ZoomOut),
                Some((_, "fit_to_window")) => Some(ViewAction::Fit),
                Some((_, "actual_size")) => Some(ViewAction::ActualSize),
                Some((_, "extract_text")) => Some(ViewAction::ExtractText),
                Some((_, "lift_subject")) => Some(ViewAction::LiftSubject),
                _ => None,
            },
        };
        if let Some(action) = action {
            self.apply_action(action);
        }
    }

    fn apply_action(&mut self, action: ViewAction) {
        match action {
            ViewAction::Open => self.open_action(),
            ViewAction::ZoomIn => self.change_zoom(1.25),
            ViewAction::ZoomOut => self.change_zoom(0.8),
            ViewAction::Fit => self.set_zoom_mode(ZoomMode::Fit),
            ViewAction::ActualSize => {
                self.zoom_mode = ZoomMode::Manual;
                self.zoom = 1.0;
                self.reflow_image();
                self.set_status("Actual size (100%)");
            }
            ViewAction::ExtractText => self.submit_vision_job(VisionAction::ExtractText),
            ViewAction::LiftSubject => self.submit_vision_job(VisionAction::LiftSubject),
        }
    }

    fn submit_vision_job(&mut self, action: VisionAction) {
        let submission_id = self.begin_vision_submission();
        let request = match self.build_vision_request(action) {
            Ok(request) => request,
            Err(error) => {
                self.active_vision_submission = None;
                self.set_status(format!("Vision unavailable: {error}; no output produced."));
                return;
            }
        };

        let mut config = match VisionClientConfig::from_environment() {
            Ok(config) => config,
            Err(error) => {
                self.active_vision_submission = None;
                self.set_status(format!("Vision unavailable: {error}; no output produced."));
                return;
            }
        };
        // Preview actions must not leave the native UI blocked for the full
        // daemon defaults when the optional session service is absent.
        config.connect_timeout = Duration::from_millis(300);
        config.write_timeout = Duration::from_millis(500);
        config.read_timeout = Duration::from_secs(2);
        let client = match VisionClient::with_config(config) {
            Ok(client) => client,
            Err(error) => {
                self.active_vision_submission = None;
                self.set_status(format!("Vision unavailable: {error}; no output produced."));
                return;
            }
        };

        let operation_name = action.label();
        match client.submit(request) {
            Ok(accepted) => {
                let watch = VisionJobWatch {
                    submission_id,
                    action,
                    job_id: accepted.job_id.clone(),
                };
                self.set_status(format!(
                    "{operation_name} submitted ({:?}); waiting for result.",
                    accepted.status
                ));
                if let Err(error) = spawn_vision_job_watcher(
                    client,
                    self.vision_event_tx.clone(),
                    self.vision_event_waker.clone(),
                    Arc::clone(&self.vision_submission_generation),
                    watch,
                ) {
                    self.active_vision_submission = None;
                    self.set_status(format!(
                        "{operation_name} accepted ({:?}) but could not be watched: {error}; no output produced.",
                        accepted.status
                    ));
                }
            }
            Err(error) => {
                self.active_vision_submission = None;
                self.set_status(format!(
                    "{operation_name} unavailable: {error}; no output produced."
                ));
            }
        }
    }

    fn begin_vision_submission(&mut self) -> u64 {
        let submission_id = self
            .vision_submission_generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1);
        self.active_vision_submission = Some(submission_id);
        submission_id
    }

    fn invalidate_vision_submission(&mut self) {
        self.vision_submission_generation
            .fetch_add(1, Ordering::AcqRel);
        self.active_vision_submission = None;
    }

    fn handle_vision_event(&mut self, event: VisionJobEvent) {
        if self.active_vision_submission != Some(event.submission_id()) {
            return;
        }

        match event {
            VisionJobEvent::Status { action, status, .. } => self.set_status(format!(
                "{} submitted ({status:?}); waiting for result.",
                action.label()
            )),
            VisionJobEvent::Terminal { result, .. } => {
                let VisionJobTerminal {
                    action,
                    job_id,
                    job,
                    asset,
                    asset_error,
                } = *result;
                self.active_vision_submission = None;
                if job.job_id != job_id {
                    self.set_status(format!(
                        "{} returned a result for an unexpected job; no output produced.",
                        action.label()
                    ));
                    return;
                }
                self.resolve_vision_job(action, job, asset, asset_error);
            }
            VisionJobEvent::Timeout {
                action,
                elapsed,
                last_error,
                ..
            } => {
                self.active_vision_submission = None;
                let detail = last_error
                    .map(|error| format!("; last poll error: {error}"))
                    .unwrap_or_default();
                self.set_status(format!(
                    "{} timed out after {} seconds while waiting for the daemon{detail}; no output produced.",
                    action.label(),
                    elapsed.as_secs().max(1)
                ));
            }
        }
    }

    fn poll_vision_events(&mut self) {
        while let Ok(event) = self.vision_event_rx.try_recv() {
            self.handle_vision_event(event);
        }
    }

    fn build_vision_request(&self, action: VisionAction) -> Result<SubmitJobRequest, String> {
        let image = self
            .image_canvas_image()
            .ok_or_else(|| "no image is loaded".to_string())?;
        let media_type = image
            .media_type
            .ok_or_else(|| "this image format is not accepted by Vision".to_string())?;
        let label = safe_file_label(&image.path);
        let source = ImageSource::Inline(InlineImage {
            metadata: ImageMetadata {
                media_type,
                encoded_bytes: image.encoded.len() as u64,
                dimensions: PixelSize {
                    width: image.source_width,
                    height: image.source_height,
                },
                sha256: None,
                label,
            },
            bytes: image.encoded.clone(),
        });
        let job = match action {
            VisionAction::ExtractText => VisionJob::ExtractText(ExtractTextJob {
                source,
                options: ExtractTextOptions::default(),
            }),
            VisionAction::LiftSubject => VisionJob::LiftSubject(LiftSubjectJob {
                source,
                options: LiftSubjectOptions::default(),
            }),
        };
        let request = SubmitJobRequest {
            client_request_id: Some(ClientRequestId(format!(
                "preview-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ))),
            job,
        };
        request
            .validate(MAX_ENCODED_BYTES, MAX_SOURCE_PIXELS)
            .map_err(|error| format!("request validation failed: {error:?}"))?;
        Ok(request)
    }

    fn resolve_vision_job(
        &mut self,
        action: VisionAction,
        job: JobResultResponse,
        asset: Option<AssetDataResponse>,
        asset_error: Option<String>,
    ) {
        let operation_name = action.label();
        if let Some(error) = job.error.as_ref() {
            let state = match job.status {
                JobStatus::Cancelled => "cancelled",
                JobStatus::Rejected => "rejected",
                _ => "failed",
            };
            self.set_status(format!(
                "{operation_name} {state}: {}; no output produced.",
                error.message
            ));
            return;
        }
        match job.status {
            JobStatus::Failed => {
                self.set_status(format!(
                    "{operation_name} failed (daemon reported Failed); no output produced."
                ));
                return;
            }
            JobStatus::Cancelled => {
                self.set_status(format!(
                    "{operation_name} cancelled by the daemon; no output produced."
                ));
                return;
            }
            JobStatus::Rejected => {
                self.set_status(format!(
                    "{operation_name} was rejected by the daemon; no output produced."
                ));
                return;
            }
            JobStatus::Queued | JobStatus::Running => {
                self.set_status(format!(
                    "{operation_name} submitted ({:?}); waiting for result.",
                    job.status
                ));
                return;
            }
            JobStatus::Succeeded => {}
        }

        let source_path = self.image_canvas_image().map(|image| image.path.clone());
        match (action, job.result) {
            (VisionAction::ExtractText, Some(VisionResult::ExtractText(result))) => {
                let Some(source_path) = source_path.as_deref() else {
                    self.set_status(
                        "Extract Text completed but no source image is loaded; no output produced.",
                    );
                    return;
                };
                match persist_extracted_text(source_path, &result.full_text) {
                    Ok(path) => self.set_status(format!(
                        "Extract Text completed: {} characters across {} lines. Saved to {}.",
                        result.full_text.chars().count(),
                        result.lines.len(),
                        path.display()
                    )),
                    Err(error) => self.set_status(format!(
                        "Extract Text completed but its output could not be saved: {error}; no output produced."
                    )),
                }
            }
            (VisionAction::LiftSubject, Some(VisionResult::LiftSubject(result))) => {
                if result.cutout.role != ArtifactRole::LiftedSubject {
                    self.set_status(
                        "Lift Subject returned an unexpected artifact role; no output produced.",
                    );
                    return;
                }
                let Some(source_path) = source_path.as_deref() else {
                    self.set_status(
                        "Lift Subject completed but no source image is loaded; no output produced.",
                    );
                    return;
                };
                let Some(asset) = asset else {
                    let detail = asset_error.unwrap_or_else(|| {
                        "the daemon did not return the lifted-subject asset".to_string()
                    });
                    self.set_status(format!(
                        "Lift Subject completed but its asset could not be retrieved: {detail}; no output produced."
                    ));
                    return;
                };
                match persist_lifted_subject(source_path, &asset) {
                    Ok((path, dimensions)) => self.set_status(format!(
                        "Lift Subject completed: {} x {} PNG saved to {}.",
                        dimensions.width,
                        dimensions.height,
                        path.display()
                    )),
                    Err(error) => self.set_status(format!(
                        "Lift Subject returned an invalid or unsaved asset: {error}; no output produced."
                    )),
                }
            }
            _ => self.set_status(format!(
                "{operation_name} completed without a usable result; no output produced."
            )),
        }
    }

    fn image_canvas_image(&self) -> Option<&ImageCanvas> {
        let content = self.image_scroll.content.as_ref()?;
        let canvas = content.as_any().downcast_ref::<ImageCanvas>()?;
        (!canvas.encoded.is_empty()).then_some(canvas)
    }

    fn open_action(&mut self) {
        let Some(path) = self.source_path.clone() else {
            self.set_status(
                "Open unavailable: this SDK has no file-picker API; launch Preview with an image path.",
            );
            return;
        };
        self.load_path(path);
    }

    fn load_path(&mut self, path: PathBuf) {
        self.invalidate_vision_submission();
        let filename = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.source_path = Some(path.clone());
        match load_image(&path) {
            Ok(image) => {
                self.source_dimensions = Some((image.width, image.height));
                self.image_scroll
                    .set_content(Box::new(ImageCanvas::from_loaded(image)));
                self.empty_hint.text.clear();
                self.zoom_mode = ZoomMode::Fit;
                self.zoom = 1.0;
                self.image_scroll.scroll_x = 0.0;
                self.image_scroll.scroll_y = 0.0;
                self.set_status(format!(
                    "Loaded {filename} — Vision uses the local daemon when available."
                ));
            }
            Err(error) => {
                self.source_dimensions = None;
                self.image_scroll
                    .set_content(Box::new(ImageCanvas::empty()));
                self.empty_hint.text = "No image loaded".to_string();
                self.zoom_mode = ZoomMode::Fit;
                self.zoom = 1.0;
                self.image_scroll.scroll_x = 0.0;
                self.image_scroll.scroll_y = 0.0;
                self.set_status(format!("Cannot open {filename}: {error}"));
            }
        }
        self.update_labels();
        self.reflow_image();
    }

    fn change_zoom(&mut self, factor: f32) {
        if self.source_dimensions.is_none() {
            self.set_status("Zoom unavailable: no image is loaded.");
            return;
        }
        self.zoom_mode = ZoomMode::Manual;
        self.zoom = clamp_zoom(self.zoom * factor);
        self.reflow_image();
        self.set_status(format!("Zoom set to {}%", zoom_percent(self.zoom)));
    }

    fn set_zoom_mode(&mut self, mode: ZoomMode) {
        self.zoom_mode = mode;
        if mode == ZoomMode::Fit {
            if let Some((width, height)) = self.source_dimensions {
                let rect = self.image_scroll.rect();
                self.zoom = fit_zoom(
                    width,
                    height,
                    (rect.width - SCROLLBAR_WIDTH - 4.0).max(1.0),
                    (rect.height - 4.0).max(1.0),
                );
            }
            self.image_scroll.scroll_x = 0.0;
            self.image_scroll.scroll_y = 0.0;
            self.reflow_image();
            self.set_status(format!("Fit to window — {}%", zoom_percent(self.zoom)));
        }
    }

    fn set_status(&mut self, status: impl Into<String>) {
        self.status_text = status.into();
        self.status_label.text = self.status_text.clone();
        self.update_labels();
    }

    fn update_labels(&mut self) {
        self.dimensions_label.text = match self.source_dimensions {
            Some((width, height)) => format!("{width} x {height}"),
            None => "No image loaded".to_string(),
        };
        self.zoom_label.text = match self.source_dimensions {
            Some((width, height)) => {
                let offset = self.image_scroll.scroll_offset();
                format!(
                    "{width} x {height}  |  {}%  |  scroll {:.0}, {:.0}",
                    zoom_percent(self.zoom),
                    offset.x,
                    offset.y
                )
            }
            None => "No image".to_string(),
        };
        self.status_label.text = self.status_text.clone();
    }

    fn reflow_image(&mut self) {
        let rect = self.rect();
        if rect.width > 0.0 && rect.height > 0.0 {
            let _ = self.layout(LayoutConstraint::tight(Size::new(rect.width, rect.height)));
        }
    }

    fn image_canvas_mut(&mut self) -> Option<&mut ImageCanvas> {
        self.image_scroll
            .content
            .as_mut()?
            .as_any_mut()
            .downcast_mut::<ImageCanvas>()
    }

    fn sync_scrollbar(&mut self) {
        if let Some(rect) = self.image_scroll.scrollbar_rect() {
            self.scrollbar.set_rect(rect);
        } else {
            self.scrollbar.set_rect(Rect::ZERO);
        }
    }

    fn dispatch_button(
        button: &mut Button,
        action: ViewAction,
        event: &Event,
    ) -> Option<(EventResult, ViewAction)> {
        let result = button.handle_event(event);
        if button.take_clicked() {
            Some((EventResult::Handled, action))
        } else if matches!(result, EventResult::Ignored) {
            None
        } else {
            Some((result, action))
        }
    }
}

impl Widget for PreviewView {
    fn widget_state(&self) -> &WidgetState {
        &self.state
    }

    fn widget_state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }

    fn layout(&mut self, constraint: LayoutConstraint) -> Size {
        let size = constraint.clamp(Size::new(constraint.max_width, constraint.max_height));
        let rect = Rect::new(self.rect().x, self.rect().y, size.width, size.height);
        self.set_rect(rect);

        let pad = 12.0;
        let toolbar_height = 38.0;
        let status_height = 25.0;
        let content_width = (rect.width - pad * 2.0).max(0.0);
        let image_rect = Rect::new(
            rect.x + pad,
            rect.y + toolbar_height + 4.0,
            content_width,
            (rect.height - toolbar_height - status_height - 4.0).max(0.0),
        );

        self.background.set_rect(rect);
        self.toolbar_background
            .set_rect(Rect::new(rect.x, rect.y, rect.width, toolbar_height));
        self.status_background.set_rect(Rect::new(
            rect.x,
            rect.y + rect.height - status_height,
            rect.width,
            status_height,
        ));

        self.filename_label
            .set_rect(Rect::new(rect.x + pad, rect.y + 8.0, 180.0, 20.0));
        self.dimensions_label
            .set_rect(Rect::new(rect.x + 195.0, rect.y + 8.0, 120.0, 20.0));
        self.zoom_label
            .set_rect(Rect::new(rect.x + 325.0, rect.y + 8.0, 260.0, 20.0));

        let mut button_x = rect.x + 12.0;
        let button_y = rect.y + 39.0;
        let button_gap = 5.0;
        for (button, width) in [
            (&mut self.open_button, 74.0),
            (&mut self.zoom_out_button, 32.0),
            (&mut self.zoom_in_button, 32.0),
            (&mut self.fit_button, 48.0),
            (&mut self.actual_size_button, 58.0),
            (&mut self.extract_text_button, 102.0),
            (&mut self.lift_subject_button, 100.0),
        ] {
            button.set_rect(Rect::new(button_x, button_y, width, 28.0));
            let _ = button.layout(LayoutConstraint::tight(Size::new(width, 28.0)));
            button_x += width + button_gap;
        }

        self.image_scroll.set_rect(image_rect);
        if self.zoom_mode == ZoomMode::Fit {
            if let Some((width, height)) = self.source_dimensions {
                self.zoom = fit_zoom(
                    width,
                    height,
                    (image_rect.width - SCROLLBAR_WIDTH - 4.0).max(1.0),
                    (image_rect.height - 4.0).max(1.0),
                );
            }
        }
        let zoom = self.zoom;
        if let Some(canvas) = self.image_canvas_mut() {
            canvas.set_zoom(zoom);
        }
        let _ = self.image_scroll.layout(LayoutConstraint::tight(Size::new(
            image_rect.width,
            image_rect.height,
        )));
        self.sync_scrollbar();

        self.empty_hint.set_rect(Rect::new(
            image_rect.x + 16.0,
            image_rect.y + (image_rect.height * 0.5).max(20.0),
            (image_rect.width - 32.0).max(0.0),
            20.0,
        ));
        self.status_label.set_rect(Rect::new(
            rect.x + pad,
            rect.y + rect.height - status_height + 3.0,
            (rect.width - pad * 2.0).max(0.0),
            20.0,
        ));
        self.update_labels();
        size
    }

    fn draw(&self, _theme: &ThemeContext) {}

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::KeyDown { key, modifiers } = event {
            let action = match (key, modifiers.meta) {
                (KeyCode::O, true) => Some(ViewAction::Open),
                (KeyCode::Equals, true) | (KeyCode::Equals, false) => Some(ViewAction::ZoomIn),
                (KeyCode::Minus, true) | (KeyCode::Minus, false) => Some(ViewAction::ZoomOut),
                (KeyCode::Key0, true) | (KeyCode::Key0, false) => Some(ViewAction::ActualSize),
                (KeyCode::F, false) => Some(ViewAction::Fit),
                _ => None,
            };
            if let Some(action) = action {
                self.apply_action(action);
                return EventResult::Handled;
            }
        }

        let mut result = self.image_scroll.handle_event(event);
        let mut clicked_action = None;
        if matches!(result, EventResult::Ignored) {
            for (button, action) in [
                (&mut self.lift_subject_button, ViewAction::LiftSubject),
                (&mut self.extract_text_button, ViewAction::ExtractText),
                (&mut self.actual_size_button, ViewAction::ActualSize),
                (&mut self.fit_button, ViewAction::Fit),
                (&mut self.zoom_in_button, ViewAction::ZoomIn),
                (&mut self.zoom_out_button, ViewAction::ZoomOut),
                (&mut self.open_button, ViewAction::Open),
            ] {
                if let Some((button_result, button_action)) =
                    Self::dispatch_button(button, action, event)
                {
                    result = button_result;
                    if matches!(result, EventResult::Handled)
                        && matches!(event, Event::MouseUp { .. } | Event::KeyDown { .. })
                    {
                        clicked_action = Some(button_action);
                    }
                    break;
                }
            }
        }
        if let Some(action) = clicked_action {
            self.apply_action(action);
        }
        self.update_labels();
        self.sync_scrollbar();
        result
    }

    fn update(&mut self) {
        self.poll_vision_events();
        self.image_scroll.update();
        self.update_labels();
        self.sync_scrollbar();
    }

    fn accessibility(&self) -> Option<AccessibilityNode> {
        Some(AccessibilityNode::new(AccessibilityRole::Window, "Preview"))
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![
            &self.background,
            &self.toolbar_background,
            &self.filename_label,
            &self.dimensions_label,
            &self.zoom_label,
            &self.open_button,
            &self.zoom_out_button,
            &self.zoom_in_button,
            &self.fit_button,
            &self.actual_size_button,
            &self.extract_text_button,
            &self.lift_subject_button,
            &self.image_scroll,
            &self.scrollbar,
            &self.empty_hint,
            &self.status_background,
            &self.status_label,
        ]
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        vec![
            &mut self.background,
            &mut self.toolbar_background,
            &mut self.filename_label,
            &mut self.dimensions_label,
            &mut self.zoom_label,
            &mut self.open_button,
            &mut self.zoom_out_button,
            &mut self.zoom_in_button,
            &mut self.fit_button,
            &mut self.actual_size_button,
            &mut self.extract_text_button,
            &mut self.lift_subject_button,
            &mut self.image_scroll,
            &mut self.scrollbar,
            &mut self.empty_hint,
            &mut self.status_background,
            &mut self.status_label,
        ]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Drop for PreviewView {
    fn drop(&mut self) {
        self.vision_submission_generation
            .fetch_add(1, Ordering::AcqRel);
    }
}

fn zoom_percent(zoom: f32) -> u32 {
    (clamp_zoom(zoom) * 100.0).round() as u32
}

fn safe_file_label(path: &Path) -> Option<FileLabel> {
    let stem = path.file_stem()?.to_str()?.to_string();
    if stem.is_empty() || stem.len() > MAX_FILE_STEM_LEN {
        return None;
    }
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    let label = FileLabel { stem, extension };
    label.validate().ok().map(|_| label)
}

fn safe_artifact_stem(source: &Path) -> String {
    let raw = source
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_string());
    let mut stem = String::new();
    for character in raw.chars() {
        let safe_character = if character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
        {
            character
        } else {
            '_'
        };
        if stem.len() + safe_character.len_utf8() > MAX_ARTIFACT_STEM_BYTES {
            break;
        }
        stem.push(safe_character);
    }
    if stem.is_empty() {
        "image".to_string()
    } else {
        stem
    }
}

fn preview_output_root_from_base(base: &Path) -> Result<PathBuf, String> {
    if !base.is_absolute() {
        return Err("Preview output base must be an absolute path".to_string());
    }
    Ok(base.join("slopos-i").join("preview").join("vision"))
}

fn preview_output_root() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .or_else(|| std::env::var_os("XDG_CACHE_HOME"))
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join(".local/share").into_os_string())
        })
        .ok_or_else(|| "no XDG data/cache base is available for Preview output".to_string())?;
    preview_output_root_from_base(Path::new(&base))
}

fn validate_artifact_component(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(format!("invalid artifact {label}"));
    }
    Ok(())
}

fn artifact_file_name(
    source: &Path,
    kind: &str,
    extension: &str,
    nonce: &str,
) -> Result<String, String> {
    validate_artifact_component(kind, "kind")?;
    validate_artifact_component(extension, "extension")?;
    validate_artifact_component(nonce, "nonce")?;

    let file_name = format!(
        "{}-vision-{kind}-{nonce}.{extension}",
        safe_artifact_stem(source)
    );
    if Path::new(&file_name)
        .file_name()
        .and_then(|name| name.to_str())
        != Some(file_name.as_str())
    {
        return Err("artifact name must remain a single path component".to_string());
    }
    Ok(file_name)
}

fn next_artifact_nonce() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{}-{timestamp}-{sequence}", std::process::id())
}

fn ensure_output_dir(root: &Path) -> Result<(), String> {
    if !root.is_absolute() {
        return Err("Preview output directory must be absolute".to_string());
    }
    fs::create_dir_all(root)
        .map_err(|error| format!("cannot create Preview output directory: {error}"))?;
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("cannot inspect Preview output directory: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Preview output directory is not an owned regular directory".to_string());
    }
    Ok(())
}

fn atomic_write_bytes_at(
    root: &Path,
    source: &Path,
    kind: &str,
    extension: &str,
    bytes: &[u8],
    max_bytes: u64,
) -> Result<PathBuf, String> {
    if bytes.len() as u64 > max_bytes {
        return Err(format!(
            "artifact is too large ({} bytes; limit is {} bytes)",
            bytes.len(),
            max_bytes
        ));
    }
    ensure_output_dir(root)?;

    for _ in 0..MAX_OUTPUT_COLLISION_ATTEMPTS {
        let nonce = next_artifact_nonce();
        let file_name = artifact_file_name(source, kind, extension, &nonce)?;
        let destination = root.join(&file_name);
        let temporary = root.join(format!(".{file_name}.tmp"));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("cannot create atomic Preview output: {error}"));
            }
        };

        let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temporary);
            return Err(format!("cannot flush atomic Preview output: {error}"));
        }

        // Linking a complete, synced temporary file creates the final name
        // atomically without replacing an existing artifact on collision.
        match fs::hard_link(&temporary, &destination) {
            Ok(()) => {
                let _ = fs::remove_file(&temporary);
                return Ok(destination);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temporary);
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(format!("cannot commit atomic Preview output: {error}"));
            }
        }
    }

    Err("could not allocate a collision-free Preview output name".to_string())
}

fn persist_extracted_text_at(root: &Path, source: &Path, text: &str) -> Result<PathBuf, String> {
    atomic_write_bytes_at(
        root,
        source,
        "text",
        "txt",
        text.as_bytes(),
        MAX_EXTRACTED_TEXT_BYTES as u64,
    )
}

fn persist_extracted_text(source: &Path, text: &str) -> Result<PathBuf, String> {
    let root = preview_output_root()?;
    persist_extracted_text_at(&root, source, text)
}

fn persist_lifted_subject_at(
    root: &Path,
    source: &Path,
    asset: &AssetDataResponse,
) -> Result<(PathBuf, PixelSize), String> {
    let dimensions = validate_asset_response(asset)?;
    let path = atomic_write_bytes_at(
        root,
        source,
        "subject",
        "png",
        &asset.bytes,
        MAX_OUTPUT_BYTES,
    )?;
    Ok((path, dimensions))
}

fn persist_lifted_subject(
    source: &Path,
    asset: &AssetDataResponse,
) -> Result<(PathBuf, PixelSize), String> {
    let root = preview_output_root()?;
    persist_lifted_subject_at(&root, source, asset)
}

fn validate_asset_response(asset: &AssetDataResponse) -> Result<PixelSize, String> {
    asset
        .asset
        .validate(MAX_OUTPUT_BYTES, MAX_SOURCE_PIXELS)
        .map_err(|error| format!("asset metadata validation failed: {error:?}"))?;
    if asset.bytes.len() as u64 != asset.asset.metadata.encoded_bytes {
        return Err("asset byte count does not match its metadata".to_string());
    }
    if asset.asset.metadata.media_type != ImageMediaType::Png {
        return Err("lifted-subject asset is not declared as PNG".to_string());
    }
    let (decoded, media_type) = decode_encoded_image_limited(&asset.bytes)?;
    let dimensions = asset.asset.metadata.dimensions;
    if decoded.width() != dimensions.width || decoded.height() != dimensions.height {
        return Err("asset dimensions do not match its metadata".to_string());
    }
    if media_type != Some(ImageMediaType::Png) {
        return Err("asset format does not match its metadata".to_string());
    }
    Ok(dimensions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use slopos_vision_protocol::{AssetId, StoredImage};

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(width, height, image::Rgba([10, 20, 30, 255]));
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(
                image.as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        bytes
    }

    #[test]
    fn vision_poll_state_keeps_nonterminal_jobs_pending() {
        let started_at = Instant::now();
        let now = started_at + Duration::from_secs(1);

        assert_eq!(
            vision_poll_state(JobStatus::Queued, started_at, now, VISION_JOB_TIMEOUT),
            VisionPollState::Pending
        );
        assert_eq!(
            vision_poll_state(JobStatus::Running, started_at, now, VISION_JOB_TIMEOUT),
            VisionPollState::Pending
        );
    }

    #[test]
    fn vision_poll_state_recognizes_terminal_success_failure_and_cancellation() {
        let started_at = Instant::now();
        let now = started_at + VISION_JOB_TIMEOUT;

        for status in [
            JobStatus::Succeeded,
            JobStatus::Failed,
            JobStatus::Cancelled,
            JobStatus::Rejected,
        ] {
            assert_eq!(
                vision_poll_state(status, started_at, now, VISION_JOB_TIMEOUT),
                VisionPollState::Terminal
            );
        }
    }

    #[test]
    fn vision_poll_state_times_out_nonterminal_jobs_at_the_bound() {
        let started_at = Instant::now();
        let before_timeout = started_at + VISION_JOB_TIMEOUT - Duration::from_millis(1);
        let at_timeout = started_at + VISION_JOB_TIMEOUT;

        assert_eq!(
            vision_poll_state(
                JobStatus::Running,
                started_at,
                before_timeout,
                VISION_JOB_TIMEOUT
            ),
            VisionPollState::Pending
        );
        assert_eq!(
            vision_poll_state(
                JobStatus::Running,
                started_at,
                at_timeout,
                VISION_JOB_TIMEOUT
            ),
            VisionPollState::TimedOut
        );
    }

    #[test]
    fn zoom_is_finite_and_bounded() {
        assert_eq!(clamp_zoom(-1.0), MIN_ZOOM);
        assert_eq!(clamp_zoom(100.0), MAX_ZOOM);
        assert_eq!(clamp_zoom(f32::INFINITY), MAX_ZOOM);
        assert_eq!(clamp_zoom(f32::NEG_INFINITY), MIN_ZOOM);
        assert_eq!(clamp_zoom(f32::NAN), MIN_ZOOM);
    }

    #[test]
    fn fit_zoom_preserves_aspect_ratio_and_bounds_extremes() {
        assert!((fit_zoom(1600, 800, 800.0, 400.0) - 0.5).abs() < f32::EPSILON);
        assert_eq!(fit_zoom(1, 1, 10_000.0, 10_000.0), MAX_ZOOM);
        assert_eq!(fit_zoom(40_000_000, 1, 1.0, 1.0), MIN_ZOOM);
        assert_eq!(fit_zoom(0, 100, 100.0, 100.0), 1.0);
    }

    #[test]
    fn path_validation_rejects_directories_and_large_encoded_files() {
        let directory = tempfile::tempdir().unwrap();
        assert!(validate_image_path(directory.path(), 1024)
            .unwrap_err()
            .contains("regular file"));

        let path = directory.path().join("large.bin");
        fs::write(&path, [0u8; 8]).unwrap();
        assert!(validate_image_path(&path, 7)
            .unwrap_err()
            .contains("too large"));
    }

    #[test]
    fn valid_image_is_dimension_checked_and_downsampled() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sample.png");
        fs::write(&path, png_bytes(128, 80)).unwrap();

        let image = load_image(&path).unwrap();
        assert_eq!((image.width, image.height), (128, 80));
        assert!(image.pixel_width <= THUMBNAIL_MAX_WIDTH);
        assert!(image.pixel_height <= THUMBNAIL_MAX_HEIGHT);
        assert_eq!(
            image.pixels.len(),
            (image.pixel_width * image.pixel_height) as usize
        );
        assert_eq!(image.path, path);
    }

    #[test]
    fn preview_output_names_are_flat_and_rooted() {
        let source = Path::new("../../unsafe name/subject?.png");
        let file_name = artifact_file_name(source, "text", "txt", "nonce_1").unwrap();
        assert_eq!(file_name, "subject_-vision-text-nonce_1.txt");
        assert!(!file_name.contains(".."));
        assert!(!file_name.contains('/'));

        let root = Path::new("/tmp/slopos-preview-output");
        let output = root.join(&file_name);
        assert_eq!(output.parent(), Some(root));
        assert!(artifact_file_name(source, "../escape", "txt", "nonce").is_err());
        assert!(preview_output_root_from_base(Path::new("relative")).is_err());
        assert_eq!(
            preview_output_root_from_base(root).unwrap(),
            root.join("slopos-i/preview/vision")
        );
    }

    #[test]
    fn atomic_output_is_bounded_collision_free_and_complete() {
        let directory = tempfile::tempdir().unwrap();
        let source = Path::new("photo.png");
        let first =
            atomic_write_bytes_at(directory.path(), source, "text", "txt", b"first", 64).unwrap();
        let second =
            atomic_write_bytes_at(directory.path(), source, "text", "txt", b"second", 64).unwrap();

        assert_ne!(first, second);
        assert_eq!(fs::read(&first).unwrap(), b"first");
        assert_eq!(fs::read(&second).unwrap(), b"second");
        assert!(fs::read_dir(directory.path()).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with('.')));
        assert!(
            atomic_write_bytes_at(directory.path(), source, "text", "txt", &[0; 65], 64,)
                .unwrap_err()
                .contains("too large")
        );
    }

    #[test]
    fn lifted_subject_validates_png_metadata_and_dimensions_before_write() {
        let directory = tempfile::tempdir().unwrap();
        let bytes = png_bytes(4, 3);
        let asset = AssetDataResponse {
            asset: StoredImage {
                asset_id: AssetId("asset-test".to_string()),
                metadata: ImageMetadata {
                    media_type: ImageMediaType::Png,
                    encoded_bytes: bytes.len() as u64,
                    dimensions: PixelSize {
                        width: 4,
                        height: 3,
                    },
                    sha256: None,
                    label: None,
                },
            },
            bytes,
        };

        let (path, dimensions) =
            persist_lifted_subject_at(directory.path(), Path::new("subject.png"), &asset).unwrap();
        assert_eq!(
            dimensions,
            PixelSize {
                width: 4,
                height: 3
            }
        );
        assert_eq!(fs::read(path).unwrap(), asset.bytes);

        let mut invalid = asset.clone();
        invalid.asset.metadata.dimensions = PixelSize {
            width: 8,
            height: 3,
        };
        assert!(
            persist_lifted_subject_at(directory.path(), Path::new("subject.png"), &invalid)
                .unwrap_err()
                .contains("dimensions")
        );

        invalid.asset.metadata.media_type = ImageMediaType::Jpeg;
        assert!(
            persist_lifted_subject_at(directory.path(), Path::new("subject.png"), &invalid)
                .unwrap_err()
                .contains("PNG")
        );
    }
}
