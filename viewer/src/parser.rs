use std::{
    fs::File, io::{BufRead, BufReader, Read}
};

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GaussianVertex {
    /// position of vertex
    position: [f32; 3],
    /// log-scales defining size of gaussian
    scale: [f32; 3],
    /// diffuse color (spherical harmonics)
    diffuse: [f32; 3],
    /// transparency level
    opacity: f32,
    /// quaternion rotation
    rotation: [f32; 4],
    // high-order coefficients
    // NOTE: for now, im commenting this out, because its a lot of data to add to each splat
    // spherical_harmonics: [f32; 45]
}

pub fn parse_ply_file(path: &str) -> std::io::Result<Vec<GaussianVertex>> {

    // open file
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // parse header

    let mut vertex_count: usize = 0;
    let mut properties = Vec::new();
    //let mut has_spherical_harmonic_coefficient = false;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let line = line.trim();

        // spherical harmonics are included in file
        // if line.contains("f_rest_0") {
        //     has_spherical_harmonic_coefficient = true;
        // }

        // get vertex count
        if line.starts_with("element vertex") {
            let parts: Vec<_> = line.split_whitespace().collect();
            vertex_count = parts[2].parse().unwrap();
        }

        // get properties of ply file
        if line.starts_with("property") {
            let parts: Vec<_> = line.split_whitespace().collect();
            properties.push(parts[2].to_string());
        }

        // end of header
        if line == "end_header" {
            break;
        }
    }

    let property_count = properties.len();

    // read in byte data after header
    let data_size = vertex_count * property_count * std::mem::size_of::<f32>();
    let mut raw_bytes = vec![0u8; data_size];
    reader.read_exact(&mut raw_bytes)?;

    // only working with floats, so cast the entire vector to floats
    let float_data: &[f32] = bytemuck::cast_slice(&raw_bytes);

    let mut splats: Vec<GaussianVertex> = Vec::with_capacity(vertex_count);
    
    for v in 0..vertex_count {
        let base = v * property_count;

        let mut position = [0.0f32; 3];
        position.copy_from_slice(&float_data[base..base + 3]);

        let mut scale = [0.0f32; 3];
        scale.copy_from_slice(&float_data[base + 3..base + 6]);

        let mut diffuse = [0.0f32; 3];
        diffuse.copy_from_slice(&float_data[base + 6..base + 9]);

        let opacity = float_data[base + 9];

        let mut rotation = [0.0f32; 4];
        rotation.copy_from_slice(&float_data[base + 10..base + 14]);
        // property_index += 4;

        // let mut sh = [0.0f32; 45];
        // if has_spherical_harmonic_coefficient {
        //      sh.copy_from_slice(&float_data[base + 14.. base + 45]);
        // }

        splats.push(GaussianVertex {
            position: position,
            scale: scale,
            diffuse: diffuse,
            opacity: opacity,
            rotation: rotation,
            // spherical_harmonics: sh,
        });
    }

    Ok(splats)
}

