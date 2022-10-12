//! Module holding [ModelOptions] and other helper types for [super::Display]

///
/// [ModelOptions] hold all the various settings that can impact a particular [super::Model]
/// `display_size` being set is the minimum requirement.
///
#[derive(Clone)]
pub struct ModelOptions {
    /// Specify display color ordering
    pub(crate) color_order: ColorOrder,
    /// Initial display orientation (without inverts)
    pub(crate) orientation: Orientation,
    /// Set to make display vertical refresh bottom to top
    pub(crate) invert_vertical_refresh: bool,
    /// Set to make display horizontal refresh right to left
    pub(crate) invert_horizontal_refresh: bool,
    /// Offset override function returning (w, h) offset for current
    /// display orientation if display is "clipped" and needs an offset for (e.g. Pico v1)
    pub(crate) window_offset_handler: fn(&ModelOptions) -> WindowOffsetResult,
    /// Display size (w, h) override for the display/model, (0, 0) for no override
    pub(crate) display_size: (u16, u16),
    /// Framebuffer size (w, h) override for the display/model, (0, 0) for no override
    pub(crate) framebuffer_size: (u16, u16),
    // cached offset values in case we can re-use them
    cached_offset: Option<WindowOffsetResult>,
}

impl ModelOptions {
    ///
    /// Constructs a [ModelOptions]
    /// with given display and framebuffer sizes
    ///
    pub fn with_sizes(display_size: (u16, u16), framebuffer_size: (u16, u16)) -> Self {
        Self {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_horizontal_refresh: false,
            invert_vertical_refresh: false,
            window_offset_handler: no_offset,
            display_size,
            framebuffer_size,
            cached_offset: None,
        }
    }

    ///
    /// Constructs a [ModelOptions]
    /// with given display and framebuffer sizes and provided window offset handler
    ///
    pub fn with_all(
        display_size: (u16, u16),
        framebuffer_size: (u16, u16),
        window_offset_handler: fn(&ModelOptions) -> WindowOffsetResult,
    ) -> Self {
        Self {
            color_order: ColorOrder::default(),
            orientation: Orientation::default(),
            invert_horizontal_refresh: false,
            invert_vertical_refresh: false,
            window_offset_handler,
            display_size,
            framebuffer_size,
            cached_offset: None,
        }
    }

    ///
    /// Returns MADCTL register value for given display options
    ///
    pub fn madctl(&self) -> u8 {
        let mut value = self.orientation.value_u8();
        if self.invert_vertical_refresh {
            value |= 0b0001_0000;
        }
        match self.color_order {
            ColorOrder::Rgb => {}
            ColorOrder::Bgr => value |= 0b0000_1000,
        }
        if self.invert_horizontal_refresh {
            value |= 0b0000_0100;
        }

        value
    }

    ///
    /// Returns display size based on current orientation and display options.
    /// Used by models.
    ///
    pub fn display_size(&self) -> (u16, u16) {
        Self::orient_size(self.display_size, self.orientation())
    }

    ///
    /// Returns framebuffer size based on current orientation and display options.
    /// Used by models. Uses display_size if framebuffer_size is not set.
    ///
    pub fn framebuffer_size(&self) -> (u16, u16) {
        let size = if self.framebuffer_size == (0, 0) {
            self.display_size
        } else {
            self.framebuffer_size
        };

        Self::orient_size(size, self.orientation())
    }

    ///
    /// Returns window offset (x, y) based on current orientation and display options.
    /// Used by [Display::set_address_window]
    ///
    pub fn window_offset(&mut self) -> (u16, u16) {
        if let Some(cached) = self.cached_offset {
            cached.into()
        } else {
            let result = (self.window_offset_handler)(self);

            if result.cachable {
                self.cached_offset = Some(result);
            }

            result.into()
        }
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    ///
    /// Sets the current [Orientation]
    ///
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    // Flip size according to orientation, in general
    fn orient_size(size: (u16, u16), orientation: Orientation) -> (u16, u16) {
        match orientation {
            Orientation::Portrait(_) | Orientation::PortraitInverted(_) => size,
            Orientation::Landscape(_) | Orientation::LandscapeInverted(_) => (size.1, size.0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WindowOffsetResult {
    pub x: u16,
    pub y: u16,
    pub cachable: bool,
}

impl From<(u16, u16)> for WindowOffsetResult {
    fn from(offset: (u16, u16)) -> Self {
        WindowOffsetResult {
            x: offset.0,
            y: offset.1,
            cachable: false,
        }
    }
}

impl From<WindowOffsetResult> for (u16, u16) {
    fn from(offset: WindowOffsetResult) -> Self {
        (offset.x, offset.y)
    }
}

///
/// `no_offset` is the default offset provider. It results to 0, 0 in case display_size is == framebuffer_size
/// and to framebuffer_size - display_size otherwise.
///
fn no_offset(options: &ModelOptions) -> WindowOffsetResult {
    let fb_w = options.framebuffer_size.0;
    let fb_h = options.framebuffer_size.1;
    let d_w = options.display_size.0;
    let d_h = options.display_size.1;

    let mut x = 0;
    let mut y = 0;

    if fb_w > d_w {
        x = fb_w - d_w;
    }

    if fb_h > d_w {
        y = fb_h - d_h;
    }

    WindowOffsetResult {
        x,
        y,
        cachable: true,
    }
}

///
/// Display orientation.
///
#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    /// Portrait orientation, with mirror image parameter
    Portrait(bool),
    /// Landscape orientation, with mirror image parameter
    Landscape(bool),
    /// Inverted Portrait orientation, with mirror image parameter
    PortraitInverted(bool),
    /// Inverted Lanscape orientation, with mirror image parameter
    LandscapeInverted(bool),
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait(false)
    }
}

impl Orientation {
    pub fn value_u8(&self) -> u8 {
        match self {
            Orientation::Portrait(false) => 0b0000_0000,
            Orientation::Portrait(true) => 0b0100_0000,
            Orientation::PortraitInverted(false) => 0b1100_0000,
            Orientation::PortraitInverted(true) => 0b1000_0000,
            Orientation::Landscape(false) => 0b0010_0000,
            Orientation::Landscape(true) => 0b0110_0000,
            Orientation::LandscapeInverted(false) => 0b1110_0000,
            Orientation::LandscapeInverted(true) => 0b1010_0000,
        }
    }
}

///
/// Tearing effect output setting.
///
#[derive(Copy, Clone)]
pub enum TearingEffect {
    /// Disable output.
    Off,
    /// Output vertical blanking information.
    Vertical,
    /// Output horizontal and vertical blanking information.
    HorizontalAndVertical,
}

///
/// Defines expected color component ordering, RGB or BGR
///
#[derive(Debug, Clone, Copy)]
pub enum ColorOrder {
    Rgb,
    Bgr,
}

impl Default for ColorOrder {
    fn default() -> Self {
        Self::Rgb
    }
}
