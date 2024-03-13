pub struct EncodedPrimative {
    pub indices: UnboundBufferSlice,
    pub vertices: UnboundBufferSlice,
}

pub struct UnboundBufferSlice {
    pub offset: usize,
    pub size: usize,
}

pub struct EncodedMesh {
    pub name: Option<String>,
    pub primatives: Vec<EncodedPrimative>,
}

pub struct EncodedSceneFile {
    pub buffer: Vec<u8>,
    pub meshes: Vec<EncodedMesh>,
}
