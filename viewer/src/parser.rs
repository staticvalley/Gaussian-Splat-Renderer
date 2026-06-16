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
    /// high-order coefficients
    spherical_harmonics: [f32; 45]
}

pub fn parse_ply_file(path: &str) -> std::io::Result<Vec<GaussianVertex>> {

    // open file
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // parse header

    let mut vertex_count: usize = 0;
    let mut properties = Vec::new();
    let mut has_spherical_harmonic_coefficient = false;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let line = line.trim();

        // spherical harmonics are included in file
        if line.contains("f_rest_0") {
            has_spherical_harmonic_coefficient = true;
        }

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

    /*  TODO: can use bytemuck to convert all data straight to f32s immediately and index like that
        would be quicker in the longrun
    */ 

    // read in byte data after header
    let data_size = vertex_count * property_count * std::mem::size_of::<f32>();
    let mut raw_bytes = vec![0u8; data_size];
    reader.read_exact(&mut raw_bytes)?;

    let mut splats: Vec<GaussianVertex> = Vec::with_capacity(vertex_count);
    
    for v in 0..vertex_count {
        let base = v * property_count;

        // read single f32, where i is the offset from the base of the current vertex
        let read_f32 = |i: usize| {
            let byte_offset = (base + i) * 4;
            let b = &raw_bytes[byte_offset..byte_offset + 4];
            f32::from_le_bytes(b.try_into().unwrap())
        };

        // reads n number of floats from raw file bytes, where n is the size of the array given
        let read_property = |property_array: &mut[f32], start: usize| {
            for i in 0..property_array.len() {
                property_array[i] = read_f32(start + i);
            };
        };

        let mut property_index = 0;

        let mut position = [0.0f32; 3];
        read_property(&mut position, property_index);
        property_index += 3;

        let mut scale = [0.0f32; 3];
        read_property(&mut scale, property_index);
        property_index += 3;

        let mut diffuse = [0.0f32; 3];
        read_property(&mut diffuse, property_index);
        property_index += 3;

        let opacity = read_f32(property_index);
        property_index += 1;

        let mut rotation = [0.0f32; 4];
        read_property(&mut rotation, property_index);
        property_index += 4;

        let mut sh = [0.0f32; 45];
        if has_spherical_harmonic_coefficient {
            read_property(&mut sh, property_index)
        }

        splats.push(GaussianVertex {
            position: position,
            scale: scale,
            diffuse: diffuse,
            opacity: opacity,
            rotation: rotation,
            spherical_harmonics: sh,
        });
    }

    Ok(splats)
}

