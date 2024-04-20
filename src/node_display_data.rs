use crate::graph_transfer_types::WbblWebappNodeType;

const BINARY_NODE_WIDTH: f64 = 150.0;
const BINARY_NODE_HEIGHT: f64 = 100.0;
const BUILTIN_NODE_WIDTH: f64 = 150.0;
const BUILTIN_NODE_HEIGHT: f64 = 100.0;
pub const PORT_SIZE: f64 = 14.0;
pub const HALF_PORT_SIZE: f64 = 7.0;
pub const PORT_LABEL_OFFSET: f64 = 35.0;

pub fn get_node_dimensions(
    node_type: WbblWebappNodeType,
    _in_edges_count: Option<u8>,
    _out_edges_count: Option<u8>,
) -> (f64, f64) {
    match node_type {
        WbblWebappNodeType::Output => (315.0, 315.0),
        WbblWebappNodeType::Slab => (200.0, 200.0),
        WbblWebappNodeType::Preview => (150.0, 170.0),
        WbblWebappNodeType::Add => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Subtract => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Multiply => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Divide => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Modulo => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Equal => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::NotEqual => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Less => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::LessEqual => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Greater => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::GreaterEqual => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::And => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::Or => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::ShiftLeft => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::ShiftRight => (BINARY_NODE_WIDTH, BINARY_NODE_HEIGHT),
        WbblWebappNodeType::WorldPosition => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::ClipPosition => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::WorldNormal => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::WorldBitangent => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::WorldTangent => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::TexCoord => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::TexCoord2 => (BUILTIN_NODE_WIDTH, BUILTIN_NODE_HEIGHT),
        WbblWebappNodeType::Junction => (PORT_SIZE * 5.0, PORT_SIZE * 3.0),
    }
}

pub fn get_in_port_position(node_type: WbblWebappNodeType, index: u8) -> (f64, f64) {
    match node_type {
        WbblWebappNodeType::Junction => (PORT_SIZE + HALF_PORT_SIZE, PORT_SIZE + HALF_PORT_SIZE),
        _ => (
            PORT_SIZE + HALF_PORT_SIZE,
            (index as f64) * (PORT_SIZE + HALF_PORT_SIZE) + PORT_LABEL_OFFSET,
        ),
    }
}

pub fn get_out_port_position(
    node_type: WbblWebappNodeType,
    index: u8,
    in_edges_count: Option<u8>,
    out_edges_count: Option<u8>,
) -> (f64, f64) {
    let (node_width, _) = get_node_dimensions(node_type, in_edges_count, out_edges_count);
    match node_type {
        WbblWebappNodeType::Junction => (
            node_width - PORT_SIZE - HALF_PORT_SIZE,
            PORT_SIZE + HALF_PORT_SIZE,
        ),
        _ => (
            node_width - PORT_SIZE - HALF_PORT_SIZE,
            (index as f64) * (PORT_SIZE + HALF_PORT_SIZE) + PORT_LABEL_OFFSET,
        ),
    }
}
