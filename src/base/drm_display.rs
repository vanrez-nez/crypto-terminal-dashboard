use drm::control::{connector, framebuffer, Device as ControlDevice};
use drm::Device as BasicDevice;
use gbm::{AsRaw, BufferObjectFlags, Device as GbmDevice};
use glow::HasContext;
use khronos_egl as egl;
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd};

pub struct Card(std::fs::File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for Card {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

impl BasicDevice for Card {}
impl ControlDevice for Card {}

pub struct Display {
    pub gl: glow::Context,
    pub width: u32,
    pub height: u32,
    egl_inst: egl::DynamicInstance<egl::EGL1_4>,
    egl_display: egl::Display,
    egl_surface: egl::Surface,
    #[allow(dead_code)]
    egl_context: egl::Context,
    gbm: GbmDevice<Card>,
    gbm_surface: gbm::Surface<()>,
    drm_fd: i32,
    crtc_handle: drm::control::crtc::Handle,
    connector_handle: drm::control::connector::Handle,
    mode: drm::control::Mode,
    front_bo: Option<gbm::BufferObject<()>>,
    front_fb: Option<framebuffer::Handle>,
    frame_count: u32,
}

impl Display {
    pub fn new() -> Result<Self, String> {
        println!("Initializing DRM/GBM/EGL display...\n");

        // Open DRM device
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/dri/card0")
            .map_err(|e| format!("Failed to open DRM: {}", e))?;
        let card = Card(file);

        // Find connected display
        let resources = card
            .resource_handles()
            .map_err(|e| format!("Failed to get resources: {}", e))?;

        let connector = resources
            .connectors()
            .iter()
            .find_map(|&c| {
                let conn = card.get_connector(c, false).ok()?;
                if conn.state() == connector::State::Connected {
                    Some(conn)
                } else {
                    None
                }
            })
            .ok_or("No connected display")?;

        let mode = connector.modes().first().ok_or("No display modes")?.clone();
        println!(
            "Display: {}x{} @ {}Hz",
            mode.size().0,
            mode.size().1,
            mode.vrefresh()
        );

        let encoder = card
            .get_encoder(connector.current_encoder().ok_or("No encoder")?)
            .map_err(|e| format!("Failed to get encoder: {}", e))?;
        let crtc_handle = encoder.crtc().ok_or("No CRTC")?;
        let connector_handle = connector.handle();

        let drm_fd = card.as_raw_fd();

        // Create GBM device and surface
        let gbm = GbmDevice::new(card).map_err(|e| format!("Failed to create GBM: {}", e))?;

        let gbm_surface = gbm
            .create_surface::<()>(
                mode.size().0 as u32,
                mode.size().1 as u32,
                gbm::Format::Xrgb8888,
                BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
            )
            .map_err(|e| format!("Failed to create GBM surface: {}", e))?;

        // Initialize EGL
        let egl_inst = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required() }
            .map_err(|e| format!("Failed to load EGL: {}", e))?;

        let egl_display =
            unsafe { egl_inst.get_display(gbm.as_raw() as _) }.ok_or("No EGL display")?;

        let (maj, min) = egl_inst
            .initialize(egl_display)
            .map_err(|e| format!("EGL init failed: {}", e))?;
        println!("EGL {}.{}", maj, min);

        let config = egl_inst
            .choose_first_config(
                egl_display,
                &[
                    egl::RED_SIZE,
                    8,
                    egl::GREEN_SIZE,
                    8,
                    egl::BLUE_SIZE,
                    8,
                    egl::ALPHA_SIZE,
                    8,
                    egl::DEPTH_SIZE,
                    0,
                    egl::RENDERABLE_TYPE,
                    egl::OPENGL_ES2_BIT,
                    egl::SURFACE_TYPE,
                    egl::WINDOW_BIT,
                    egl::NONE,
                ],
            )
            .map_err(|e| format!("Config error: {}", e))?
            .ok_or("No suitable EGL config")?;

        egl_inst
            .bind_api(egl::OPENGL_ES_API)
            .map_err(|e| format!("Failed to bind API: {}", e))?;

        let egl_context = egl_inst
            .create_context(
                egl_display,
                config,
                None,
                &[egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE],
            )
            .map_err(|e| format!("Context failed: {}", e))?;

        let egl_surface = unsafe {
            egl_inst.create_window_surface(egl_display, config, gbm_surface.as_raw() as _, None)
        }
        .map_err(|e| format!("Window surface failed: {}", e))?;

        egl_inst
            .make_current(
                egl_display,
                Some(egl_surface),
                Some(egl_surface),
                Some(egl_context),
            )
            .map_err(|e| format!("Make current failed: {}", e))?;

        // Create OpenGL ES context
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                egl_inst
                    .get_proc_address(s)
                    .map(|p| p as _)
                    .unwrap_or(std::ptr::null())
            })
        };

        println!("Renderer: {}", unsafe {
            gl.get_parameter_string(glow::RENDERER)
        });

        let width = mode.size().0 as u32;
        let height = mode.size().1 as u32;

        unsafe {
            gl.viewport(0, 0, width as i32, height as i32);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        Ok(Display {
            gl,
            width,
            height,
            egl_inst,
            egl_display,
            egl_surface,
            egl_context,
            gbm,
            gbm_surface,
            drm_fd,
            crtc_handle,
            connector_handle,
            mode,
            front_bo: None,
            front_fb: None,
            frame_count: 0,
        })
    }

    pub fn swap_buffers(&mut self) -> Result<(), String> {
        self.egl_inst
            .swap_buffers(self.egl_display, self.egl_surface)
            .map_err(|e| format!("Swap failed: {}", e))?;

        let bo = unsafe {
            self.gbm_surface
                .lock_front_buffer()
                .map_err(|e| format!("Lock buffer failed: {}", e))?
        };

        let fb = self
            .gbm
            .add_framebuffer(&bo, 24, 32)
            .map_err(|e| format!("Failed to create framebuffer: {}", e))?;

        if self.frame_count == 0 {
            self.gbm
                .set_crtc(
                    self.crtc_handle,
                    Some(fb),
                    (0, 0),
                    &[self.connector_handle],
                    Some(self.mode.clone()),
                )
                .map_err(|e| format!("Failed to set CRTC: {}", e))?;
        } else {
            self.gbm
                .page_flip(
                    self.crtc_handle,
                    fb,
                    drm::control::PageFlipFlags::EVENT,
                    None,
                )
                .map_err(|e| format!("Page flip failed: {}", e))?;

            let mut fds = [libc::pollfd {
                fd: self.drm_fd,
                events: libc::POLLIN,
                revents: 0,
            }];
            unsafe {
                libc::poll(fds.as_mut_ptr(), 1, 1000);
                let mut buf = [0u8; 1024];
                libc::read(self.drm_fd, buf.as_mut_ptr() as _, buf.len());
            }
        }

        if let Some(old_fb) = self.front_fb.take() {
            let _ = self.gbm.destroy_framebuffer(old_fb);
        }
        drop(self.front_bo.take());

        self.front_bo = Some(bo);
        self.front_fb = Some(fb);
        self.frame_count += 1;

        Ok(())
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            self.gl.clear_color(r, g, b, a);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        // Cleanup must happen in correct order to avoid segfault
        if let Some(fb) = self.front_fb.take() {
            let _ = self.gbm.destroy_framebuffer(fb);
        }
        drop(self.front_bo.take());

        let _ = self
            .egl_inst
            .make_current(self.egl_display, None, None, None);
        let _ = self
            .egl_inst
            .destroy_surface(self.egl_display, self.egl_surface);
        let _ = self
            .egl_inst
            .destroy_context(self.egl_display, self.egl_context);
    }
}
