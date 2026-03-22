use renderer::render_backend::vertex::Vertex;

enum Face {
    Left,
    Right,
    Up,
    Down,
    Front,
    Back,
}

pub fn build_chunk(chunk: Box<[f64; 32768]>) -> Vec<Vertex> {
    const T: usize = 32;

    let idx = |x: usize, y: usize, z: usize| y * (T * T) + z * T + x;

    let mut render_chunk: Vec<Vertex> = Vec::with_capacity(32768 * 6 / 2);

    for i in 0..chunk.len() {
        if chunk[i] <= 0.0 {
            continue;
        }

        let x = i % T;
        let z = (i / T) % T;
        let y = i / (T * T);
        let position = (x, y, z);

        if y == 0     || chunk[idx(x, y - 1, z)] <= 0.0 { gen_face(Face::Down,  position, &mut render_chunk); }
        if y == T - 1 || chunk[idx(x, y + 1, z)] <= 0.0 { gen_face(Face::Up,    position, &mut render_chunk); }
        if z == 0     || chunk[idx(x, y, z - 1)] <= 0.0 { gen_face(Face::Front, position, &mut render_chunk); }
        if z == T - 1 || chunk[idx(x, y, z + 1)] <= 0.0 { gen_face(Face::Back,  position, &mut render_chunk); }
        if x == 0     || chunk[idx(x - 1, y, z)] <= 0.0 { gen_face(Face::Left,  position, &mut render_chunk); }
        if x == T - 1 || chunk[idx(x + 1, y, z)] <= 0.0 { gen_face(Face::Right, position, &mut render_chunk); }
    }

    render_chunk
}

fn gen_face(face: Face, position: (usize, usize, usize), out: &mut Vec<Vertex>) {
    let (px, py, pz) = (position.0 as f32, position.1 as f32, position.2 as f32);

    let [v0, v1, v2, v3] = match face {
        Face::Front => [
            Vertex { position: [-0.5 + px, -0.5 + py,  0.5 + pz] },
            Vertex { position: [ 0.5 + px, -0.5 + py,  0.5 + pz] },
            Vertex { position: [ 0.5 + px,  0.5 + py,  0.5 + pz] },
            Vertex { position: [-0.5 + px,  0.5 + py,  0.5 + pz] },
        ],
        Face::Back  => [
            Vertex { position: [ 0.5 + px, -0.5 + py, -0.5 + pz] },
            Vertex { position: [-0.5 + px, -0.5 + py, -0.5 + pz] },
            Vertex { position: [-0.5 + px,  0.5 + py, -0.5 + pz] },
            Vertex { position: [ 0.5 + px,  0.5 + py, -0.5 + pz] },
        ],
        Face::Left  => [
            Vertex { position: [-0.5 + px, -0.5 + py, -0.5 + pz] },
            Vertex { position: [-0.5 + px, -0.5 + py,  0.5 + pz] },
            Vertex { position: [-0.5 + px,  0.5 + py,  0.5 + pz] },
            Vertex { position: [-0.5 + px,  0.5 + py, -0.5 + pz] },
        ],
        Face::Right => [
            Vertex { position: [ 0.5 + px, -0.5 + py,  0.5 + pz] },
            Vertex { position: [ 0.5 + px, -0.5 + py, -0.5 + pz] },
            Vertex { position: [ 0.5 + px,  0.5 + py, -0.5 + pz] },
            Vertex { position: [ 0.5 + px,  0.5 + py,  0.5 + pz] },
        ],
        Face::Up    => [
            Vertex { position: [-0.5 + px,  0.5 + py,  0.5 + pz] },
            Vertex { position: [ 0.5 + px,  0.5 + py,  0.5 + pz] },
            Vertex { position: [ 0.5 + px,  0.5 + py, -0.5 + pz] },
            Vertex { position: [-0.5 + px,  0.5 + py, -0.5 + pz] },
        ],
        Face::Down  => [
            Vertex { position: [-0.5 + px, -0.5 + py, -0.5 + pz] },
            Vertex { position: [ 0.5 + px, -0.5 + py, -0.5 + pz] },
            Vertex { position: [ 0.5 + px, -0.5 + py,  0.5 + pz] },
            Vertex { position: [-0.5 + px, -0.5 + py,  0.5 + pz] },
        ],
    };

    out.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
}