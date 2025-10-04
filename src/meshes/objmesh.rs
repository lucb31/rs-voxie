use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ObjMesh {
    vpos: Vec<[f32; 3]>, // vertex positions
    tpos: Vec<[f32; 2]>, // texture coordinates
    norm: Vec<[f32; 3]>, // normals

    face: Vec<Vec<usize>>, // vertex indices
    tfac: Vec<Vec<usize>>, // texture coordinate indices
    nfac: Vec<Vec<usize>>, // normal indices
}

#[derive(Debug)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug)]
pub struct VertexBuffers {
    pub position_buffer: Vec<f32>,
    pub tex_coord_buffer: Vec<f32>,
    pub normal_buffer: Vec<f32>,
}

impl ObjMesh {
    pub fn new() -> Self {
        Self {
            vpos: vec![],
            tpos: vec![],
            norm: vec![],
            face: vec![],
            tfac: vec![],
            nfac: vec![],
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let contents = fs::read_to_string(path)?;
        self.parse(&contents);
        Ok(())
    }

    pub fn parse(&mut self, objdata: &str) {
        for line in objdata.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" => {
                    let x = parts[1].parse().unwrap_or(0.0);
                    let y = parts[2].parse().unwrap_or(0.0);
                    let z = parts[3].parse().unwrap_or(0.0);
                    self.vpos.push([x, y, z]);
                }
                "vt" => {
                    let u = parts[1].parse().unwrap_or(0.0);
                    let v = parts[2].parse().unwrap_or(0.0);
                    self.tpos.push([u, v]);
                }
                "vn" => {
                    let x = parts[1].parse().unwrap_or(0.0);
                    let y = parts[2].parse().unwrap_or(0.0);
                    let z = parts[3].parse().unwrap_or(0.0);
                    self.norm.push([x, y, z]);
                }
                "f" => {
                    let mut f = vec![];
                    let mut tf = vec![];
                    let mut nf = vec![];

                    for vertex in &parts[1..] {
                        let ids: Vec<&str> = vertex.split('/').collect();

                        let vid = Self::parse_index(ids.get(0), self.vpos.len());
                        f.push(vid);

                        if let Some(tid_str) = ids.get(1) {
                            if !tid_str.is_empty() {
                                let tid = Self::parse_index(Some(tid_str), self.tpos.len());
                                tf.push(tid);
                            }
                        }

                        if let Some(nid_str) = ids.get(2) {
                            if !nid_str.is_empty() {
                                let nid = Self::parse_index(Some(nid_str), self.norm.len());
                                nf.push(nid);
                            }
                        }
                    }

                    self.face.push(f);
                    if !tf.is_empty() {
                        self.tfac.push(tf);
                    }
                    if !nf.is_empty() {
                        self.nfac.push(nf);
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_index(index: Option<&&str>, len: usize) -> usize {
        if let Some(id_str) = index {
            let mut id = id_str.parse::<isize>().unwrap_or(0);
            if id < 0 {
                id = (len as isize) + id + 1;
            }
            return (id - 1) as usize;
        }
        0
    }

    pub fn get_bounding_box(&self) -> Option<BoundingBox> {
        if self.vpos.is_empty() {
            return None;
        }

        let mut min = self.vpos[0];
        let mut max = self.vpos[0];

        for v in &self.vpos[1..] {
            for i in 0..3 {
                if v[i] < min[i] {
                    min[i] = v[i];
                }
                if v[i] > max[i] {
                    max[i] = v[i];
                }
            }
        }

        Some(BoundingBox { min, max })
    }

    pub fn shift_and_scale(&mut self, shift: [f32; 3], scale: f32) {
        for v in &mut self.vpos {
            for i in 0..3 {
                v[i] = (v[i] + shift[i]) * scale;
            }
        }
    }

    pub fn get_vertex_buffers(&self) -> VertexBuffers {
        let mut v_buffer = vec![];
        let mut t_buffer = vec![];
        let mut n_buffer = vec![];

        for (i, f) in self.face.iter().enumerate() {
            if f.len() < 3 {
                continue;
            }

            self.add_triangle_to_buffers(&mut v_buffer, &mut t_buffer, &mut n_buffer, i, 0, 1, 2);
            for j in 3..f.len() {
                self.add_triangle_to_buffers(
                    &mut v_buffer,
                    &mut t_buffer,
                    &mut n_buffer,
                    i,
                    0,
                    j - 1,
                    j,
                );
            }
        }

        VertexBuffers {
            position_buffer: v_buffer,
            tex_coord_buffer: t_buffer,
            normal_buffer: n_buffer,
        }
    }

    fn add_triangle_to_buffers(
        &self,
        v_buffer: &mut Vec<f32>,
        t_buffer: &mut Vec<f32>,
        n_buffer: &mut Vec<f32>,
        fi: usize,
        i: usize,
        j: usize,
        k: usize,
    ) {
        let f = &self.face[fi];
        let tf = self.tfac.get(fi);
        let nf = self.nfac.get(fi);

        Self::add_triangle_to_buffer(v_buffer, &self.vpos, f, i, j, k, Self::add_vert_to_buffer3);

        if let Some(tf) = tf {
            Self::add_triangle_to_buffer(
                t_buffer,
                &self.tpos,
                tf,
                i,
                j,
                k,
                Self::add_vert_to_buffer2,
            );
        }

        if let Some(nf) = nf {
            Self::add_triangle_to_buffer(
                n_buffer,
                &self.norm,
                nf,
                i,
                j,
                k,
                Self::add_vert_to_buffer3,
            );
        }
    }

    fn add_triangle_to_buffer<T>(
        buffer: &mut Vec<f32>,
        v: &[T],
        f: &[usize],
        i: usize,
        j: usize,
        k: usize,
        add_vert: fn(&mut Vec<f32>, &[T], &[usize], usize),
    ) {
        add_vert(buffer, v, f, i);
        add_vert(buffer, v, f, j);
        add_vert(buffer, v, f, k);
    }

    fn add_vert_to_buffer3(buffer: &mut Vec<f32>, v: &[[f32; 3]], f: &[usize], i: usize) {
        buffer.extend_from_slice(&v[f[i]]);
    }

    fn add_vert_to_buffer2(buffer: &mut Vec<f32>, v: &[[f32; 2]], f: &[usize], i: usize) {
        buffer.extend_from_slice(&v[f[i]]);
    }
}
