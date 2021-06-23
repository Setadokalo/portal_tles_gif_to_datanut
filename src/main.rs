use std::{borrow::Borrow, collections::HashMap, env, error::Error, fmt::Display, fs::write};

use gif::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Color {
	r: u8,
	g: u8,
	b: u8,
}
impl Color {
	fn new(r: u8, g: u8, b: u8) -> Self {
		Color {r, g, b}
	}
	fn as_mixed(mut self, other: &Color) -> Self {
		self.r = self.r / 2 + other.r / 2;
		self.g = self.g / 2 + other.g / 2;
		self.b = self.b / 2 + other.b / 2;

		self
	}
	fn mix(&self, other: &Color) -> Self {
		Self::new(self.r / 2 + other.r / 2,
				self.g / 2 + other.g / 2,
				self.b / 2 + other.b / 2)
	}
}

fn simplify_palette(palette_raw: impl IntoIterator<Item = impl Borrow<u8>>) -> (Vec<u8>, HashMap<usize, usize>) {
	let palette_raw: Vec<u8> = palette_raw.into_iter().map(|b| *b.borrow()).collect();
	let mut palette: Vec<Color> = Vec::with_capacity(palette_raw.len() / 3);
	for i in 0..(palette_raw.len()/3) {
		palette.push(Color::new(palette_raw[i * 3], palette_raw[i * 3 + 1], palette_raw[i * 3 + 2]));
	}
	let color_count = palette.len();
	let mut cmap = HashMap::new();
	for i in 0..color_count {
		cmap.insert(i, i);
	}
	while palette.len() > 64 {
		let mut closest_distance = f64::MAX;
		let mut closest_index = 0;
		let mut closest_hit_idx = 0;
		for c_idx in 0..palette.len() {
			let col = &palette[c_idx];
			for nc_idx in (c_idx+1)..palette.len() {
				let ncol = &palette[nc_idx];
				let color_distance = ((col.r as f64 - ncol.r as f64).powi(2) + (col.g as f64 - ncol.g as f64).powi(2) + (col.b as f64 - col.b as f64).powi(2)).sqrt();
				if color_distance < closest_distance {
					closest_distance = color_distance;
					closest_index = c_idx;
					closest_hit_idx = nc_idx;
				}
			}
		}
		let ccol_1 = palette[closest_index];
		let ccol_2 = palette[closest_hit_idx];
		println!("Combining colors {:?} and {:?} (indexes {} and {}) into {:?}", ccol_1, ccol_2, closest_index, closest_hit_idx, ccol_1.mix(&ccol_2));
		palette[closest_index] = ccol_1.as_mixed(&ccol_2);
		for (_, value) in cmap.iter_mut() {
			if *value > closest_hit_idx {
				*value -= 1;
			}
			if *value == closest_hit_idx {
				*value = closest_index;
			}
		}
		// cmap.insert(ccol_2.0, closest_index);
		palette.remove(closest_hit_idx);
	}
	println!("Palette simplified to \nlength of {}, Map of source to target colors is {:?}", palette.len(), cmap);
	let mut conv_palette = Vec::with_capacity(palette.len() * 3);
	for color in palette {
		conv_palette.push(color.r);
		conv_palette.push(color.g);
		conv_palette.push(color.b);
	}
	(conv_palette, cmap)
}

fn palette_to_string<'a>(palette: impl IntoIterator<Item = impl Borrow<u8>>) -> (String, HashMap<usize, usize>) {
	let (palette, remap) = simplify_palette(palette);
	let mut s = String::with_capacity(palette.len());
	for (idx, c_channel) in palette.iter().enumerate() {
		s += &format!("{}, ", c_channel);
		// s += &format!("{:0>2x}", c_channel.borrow());

		if idx == 63 * 3 + 2 {
			break;
		}
	}
	(s, remap)
}

const BASE_64: [char; 64] = [
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 
	'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 
	'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 
	'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 
	'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 
	'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 
	'w', 'x', 'y', 'z', '0', '1', '2', '3', 
	'4', '5', '6', '7', '8', '9', '+', '/',
];

// #[derive(Debug)]
// struct DummyError {
// 	reason: String
// }
// impl DummyError {
// 	fn new(reason: impl Into<String>) -> Self {
// 		Self { reason: reason.into() }
// 	}
// }

// impl Display for DummyError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//       	self.reason.fmt(f)
//     }
// }

// impl Error for DummyError {}

fn main() -> anyhow::Result<()>{
	let mut encoded: String;
	let f = std::fs::File::open(env::args().nth(2).unwrap_or("convert.gif".to_string()))?;
	let mut d = Decoder::new(f)?;
	let mut frames = Vec::new();
	let mut local_palettes = false;
	let (g_palette_string, palette_map) = palette_to_string(d.palette()?);

	let prelude = format!("PALETTE <- [{}]\nDATA <- @\"", g_palette_string);
	// let mut next_frame = d.read_next_frame()?;
	while let Some(frame) = d.read_next_frame()? {
		frames.push(frame.clone());
		local_palettes = local_palettes || frame.palette.is_some();
	}

	encoded = String::with_capacity(prelude.len() + 1 + (d.width() * d.height()) as usize * frames.len());
	let o_capacity = encoded.capacity();
	if !local_palettes {
		encoded += &prelude;
	} else {
		panic!("Local Palettes are not implemented");
		// encoded = "L".to_string();
	}
	let mut f_cnt = 0;
	for frame in frames {
		// if local_palettes {
		// 	if let Some(palette) = frame.palette {
		// 		encoded += &format!("{}", palette_to_string(palette));
		// 	} else {
		// 		encoded += &g_palette_string;
		// 	}
		// }
		f_cnt += 1;
		for p in frame.buffer.iter() {
			let p = *palette_map.get(&(*p as usize)).expect("Palette did not contain entry");
			if p > 63 {
				panic!("GIF uses more than 64 palette entries! This does not encode properly! Clamping the invalid index to 63...");
			}
			encoded.push(BASE_64[p]);
		}
	}
	encoded += &format!("\"\nFRAME_COUNT <- {}", f_cnt);
	// println!("{}", encoded);
	if o_capacity != encoded.capacity() {
		eprintln!("Capacity changed during runtime!")
	}
	write("data.nut", encoded)?;
	Ok(())
}
