// port of engine/src/gpu/gx_state.cpp

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BlendFactor {
    Zero = 0,
    One,
    SourceColor,
    InverseSourceColor,
    SourceAlpha,
    InverseSourceAlpha,
    DestinationColor,
    InverseDestinationColor,
    DestinationAlpha,
    InverseDestinationAlpha,
    TevColor,
    TevAlpha,
    InverseTevAlpha,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BlendOperation {
    Add = 0,
    Subtract,
    ReverseSubtract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogicSource {
    Tev = 0,
    Zero,
    One,
    Inverted,
}

#[derive(Clone, Copy)]
pub struct BlendState {
    pub enable: bool,
    pub operation: BlendOperation,
    pub alpha_operation: BlendOperation,
    pub source: BlendFactor,
    pub destination: BlendFactor,
    pub source_alpha: BlendFactor,
    pub destination_alpha: BlendFactor,
    pub color_update: bool,
    pub alpha_update: bool,
    pub logic_op: bool,
    pub logic_mode: u32,
    pub output: LogicSource,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            enable: false,
            operation: BlendOperation::Add,
            alpha_operation: BlendOperation::Add,
            source: BlendFactor::One,
            destination: BlendFactor::Zero,
            source_alpha: BlendFactor::One,
            destination_alpha: BlendFactor::Zero,
            color_update: false,
            alpha_update: false,
            logic_op: false,
            logic_mode: 0,
            output: LogicSource::Tev,
        }
    }
}

struct LogicApproximation {
    enable: bool,
    operation: BlendOperation,
    source: BlendFactor,
    destination: BlendFactor,
    output: LogicSource,
    exact: bool,
}

const LOGIC_APPROXIMATIONS: [LogicApproximation; 16] = [
    LogicApproximation { enable: false, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::Zero, output: LogicSource::Zero, exact: true },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::DestinationColor, destination: BlendFactor::Zero, output: LogicSource::Tev, exact: false },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::InverseDestinationColor, destination: BlendFactor::Zero, output: LogicSource::Tev, exact: false },
    LogicApproximation { enable: false, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::Zero, output: LogicSource::Tev, exact: true },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::Zero, destination: BlendFactor::InverseSourceColor, output: LogicSource::Tev, exact: false },
    LogicApproximation { enable: false, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::Zero, output: LogicSource::Tev, exact: true },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::InverseDestinationColor, destination: BlendFactor::InverseSourceColor, output: LogicSource::Tev, exact: false },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::InverseSourceColor, output: LogicSource::Tev, exact: false },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::InverseDestinationColor, destination: BlendFactor::Zero, output: LogicSource::Inverted, exact: false },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::InverseDestinationColor, destination: BlendFactor::InverseSourceColor, output: LogicSource::Inverted, exact: false },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::InverseDestinationColor, destination: BlendFactor::Zero, output: LogicSource::One, exact: true },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::InverseDestinationColor, destination: BlendFactor::TevColor, output: LogicSource::One, exact: false },
    LogicApproximation { enable: false, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::Zero, output: LogicSource::Inverted, exact: true },
    LogicApproximation { enable: true, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::InverseSourceColor, output: LogicSource::Inverted, exact: false },
    LogicApproximation { enable: true, operation: BlendOperation::Subtract, source: BlendFactor::One, destination: BlendFactor::TevColor, output: LogicSource::One, exact: false },
    LogicApproximation { enable: false, operation: BlendOperation::Add, source: BlendFactor::One, destination: BlendFactor::Zero, output: LogicSource::One, exact: true },
];

const LOGIC_NAMES: [&str; 16] = [
    "clear", "and", "and_reverse", "copy", "and_inverted", "noop", "xor", "or",
    "nor", "equiv", "invert", "or_reverse", "copy_inverted", "or_inverted", "nand", "set",
];

// port of engine/src/gpu/gx_state.cpp source_factor
fn source_factor(code: u32) -> BlendFactor {
    const FACTORS: [BlendFactor; 8] = [
        BlendFactor::Zero,
        BlendFactor::One,
        BlendFactor::DestinationColor,
        BlendFactor::InverseDestinationColor,
        BlendFactor::TevAlpha,
        BlendFactor::InverseTevAlpha,
        BlendFactor::DestinationAlpha,
        BlendFactor::InverseDestinationAlpha,
    ];
    FACTORS[(code & 7) as usize]
}

// port of engine/src/gpu/gx_state.cpp destination_factor
fn destination_factor(code: u32) -> BlendFactor {
    const FACTORS: [BlendFactor; 8] = [
        BlendFactor::Zero,
        BlendFactor::One,
        BlendFactor::SourceColor,
        BlendFactor::InverseSourceColor,
        BlendFactor::TevAlpha,
        BlendFactor::InverseTevAlpha,
        BlendFactor::DestinationAlpha,
        BlendFactor::InverseDestinationAlpha,
    ];
    FACTORS[(code & 7) as usize]
}

// port of engine/src/gpu/gx_state.cpp without_destination_alpha
fn without_destination_alpha(factor: BlendFactor) -> BlendFactor {
    if factor == BlendFactor::DestinationAlpha {
        return BlendFactor::One;
    }
    if factor == BlendFactor::InverseDestinationAlpha {
        return BlendFactor::Zero;
    }
    factor
}

// port of engine/src/gpu/gx_state.cpp blend_alpha
fn blend_alpha(factor: BlendFactor) -> BlendFactor {
    match factor {
        BlendFactor::SourceColor => BlendFactor::TevAlpha,
        BlendFactor::InverseSourceColor => BlendFactor::InverseTevAlpha,
        BlendFactor::DestinationColor => BlendFactor::DestinationAlpha,
        BlendFactor::InverseDestinationColor => BlendFactor::InverseDestinationAlpha,
        _ => factor,
    }
}

// port of engine/src/gpu/gx_state.cpp logic_alpha
fn logic_alpha(factor: BlendFactor) -> BlendFactor {
    match factor {
        BlendFactor::SourceColor => BlendFactor::SourceAlpha,
        BlendFactor::InverseSourceColor => BlendFactor::InverseSourceAlpha,
        BlendFactor::DestinationColor => BlendFactor::DestinationAlpha,
        BlendFactor::InverseDestinationColor => BlendFactor::InverseDestinationAlpha,
        BlendFactor::TevColor => BlendFactor::TevAlpha,
        _ => factor,
    }
}

// port of engine/src/gpu/gx_state.cpp blend_state
pub fn blend_state(mode: u32, target_has_alpha: bool, constant_alpha: bool, alpha_test_can_pass: bool) -> BlendState {
    let mut state = BlendState::default();
    state.color_update = (mode & (1 << 3)) != 0 && alpha_test_can_pass;
    state.alpha_update = (mode & (1 << 4)) != 0 && target_has_alpha && alpha_test_can_pass;
    let constant = constant_alpha && state.alpha_update;
    if (mode & 1) != 0 && (mode & (1 << 11)) != 0 {
        state.enable = true;
        state.operation = BlendOperation::ReverseSubtract;
        state.source = BlendFactor::One;
        state.destination = BlendFactor::One;
        state.alpha_operation = if constant { BlendOperation::Add } else { BlendOperation::ReverseSubtract };
        state.source_alpha = BlendFactor::One;
        state.destination_alpha = if constant { BlendFactor::Zero } else { BlendFactor::One };
    } else if (mode & 1) != 0 {
        state.enable = true;
        let mut source = source_factor((mode >> 8) & 7);
        let mut destination = destination_factor((mode >> 5) & 7);
        if !target_has_alpha {
            source = without_destination_alpha(source);
            destination = without_destination_alpha(destination);
        }
        state.source = source;
        state.destination = destination;
        state.source_alpha = if constant { BlendFactor::One } else { blend_alpha(source) };
        state.destination_alpha = if constant { BlendFactor::Zero } else { blend_alpha(destination) };
    } else if (mode & 2) != 0 {
        state.logic_op = true;
        state.logic_mode = (mode >> 12) & 15;
        if state.logic_mode == 5 {
            state.color_update = false;
            state.alpha_update = constant;
            return state;
        }
        let approximation = &LOGIC_APPROXIMATIONS[state.logic_mode as usize];
        state.output = approximation.output;
        if approximation.enable {
            state.enable = true;
            state.operation = approximation.operation;
            state.source = approximation.source;
            state.destination = approximation.destination;
            state.alpha_operation = if constant { BlendOperation::Add } else { approximation.operation };
            state.source_alpha = if constant { BlendFactor::One } else { logic_alpha(approximation.source) };
            state.destination_alpha = if constant { BlendFactor::Zero } else { logic_alpha(approximation.destination) };
        }
    }
    state
}

// port of engine/src/gpu/gx_state.cpp logic_op_exact
pub fn logic_op_exact(logic_mode: u32) -> bool {
    LOGIC_APPROXIMATIONS[(logic_mode & 15) as usize].exact
}

// port of engine/src/gpu/gx_state.cpp logic_op_name
pub fn logic_op_name(logic_mode: u32) -> &'static str {
    LOGIC_NAMES[(logic_mode & 15) as usize]
}

#[derive(Clone, Copy, Default)]
pub struct LinePointOffsets {
    pub line: f32,
    pub point: f32,
    pub line_coordinates: u32,
    pub point_coordinates: u32,
}

// port of engine/src/gpu/gx_state.cpp line_point_offsets
pub fn line_point_offsets(bp: &[u32]) -> LinePointOffsets {
    const OFFSETS: [f32; 8] = [0.0, 1.0 / 16.0, 1.0 / 8.0, 1.0 / 4.0, 1.0 / 2.0, 1.0, 1.0, 1.0];
    let mut offsets = LinePointOffsets::default();
    offsets.line = OFFSETS[((bp[0x22] >> 16) & 7) as usize];
    offsets.point = OFFSETS[((bp[0x22] >> 19) & 7) as usize];
    for i in 0..8u32 {
        let s = bp[(0x30 + 2 * i) as usize];
        offsets.line_coordinates |= ((s >> 18) & 1) << i;
        offsets.point_coordinates |= ((s >> 19) & 1) << i;
    }
    if offsets.line == 0.0 {
        offsets.line_coordinates = 0;
    }
    if offsets.point == 0.0 {
        offsets.point_coordinates = 0;
    }
    offsets
}

// port of engine/src/gpu/gx_state.cpp line_offset_negative_side
pub fn line_offset_negative_side(dx: f32, dy: f32, tall: bool) -> bool {
    if tall { dy < 0.0 } else { dx > 0.0 }
}

#[derive(Clone, Copy)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
    pub color: [[f32; 4]; 2],
    pub uv: [[f32; 3]; 8],
}

impl Default for ScreenVertex {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0, color: [[0.0; 4]; 2], uv: [[0.0; 3]; 8] }
    }
}

// port of engine/src/gpu/gx_state.cpp offset_texture_coordinates
pub fn offset_texture_coordinates(vertex: &mut ScreenVertex, coordinates: u32, s: f32, t: f32) {
    for i in 0..8usize {
        if coordinates & (1 << i) != 0 {
            vertex.uv[i][0] += s * vertex.uv[i][2];
            vertex.uv[i][1] += t * vertex.uv[i][2];
        }
    }
}
