use std::io::Cursor;
use image::{io::Reader as ImageReader, DynamicImage};

use rustfft::{FftPlanner, num_complex::Complex, FftDirection};


fn main() {    
    println!("Hello, world!");

    let image = read_image("img/kickme.jpg".into()); //mandrill.jpg

    let (width , height) = image.dimensions();
    let mut img_buffer = dynimg2complex(image);
    fft_forward(width as usize, height as usize, &mut img_buffer);

}

pub fn fft_forward(width: usize, height: usize, img_buffer: &mut [Complex<f64>]){
    let mut planner = FftPlanner::new();
    //let fft = planner.plan_fft_forward(1234);  // desugars to:
    let fft_width = planner.plan_fft(width as usize, FftDirection::Forward);

    let mut scratch = vec![Complex::default(); fft_width.get_inplace_scratch_len()];
    for row_buffer in img_buffer.chunks_exact_mut(width as usize) {
        fft_width.process_with_scratch(row_buffer, &mut scratch);
    }

    // Transpose the image to be able to compute the FFT on the other dimension.
    let mut transposed = transpose(width, height, img_buffer);

    let fft_height = planner.plan_fft(height as usize, FftDirection::Forward);
    scratch.resize(fft_height.get_outofplace_scratch_len(), Complex::default());

    for (tr_buf, col_buf) in transposed
        .chunks_exact_mut(height as usize)
        .zip(img_buffer.chunks_exact_mut(height as usize)){            
            fft_height.process_outofplace_with_scratch(tr_buf, col_buf, &mut scratch);
    }    

}

fn read_image(file: String) -> image::GrayImage{
    let img = image::open(file).unwrap().into_luma8();
    img
}

fn dynimg2complex(img: image::GrayImage ) -> Vec<Complex<f64>>{
    let (width, height) = img.dimensions();

    // Convert the image buffer to complex numbers to be able to compute the FFT.
    let mut img_buffer: Vec<Complex<f64>> = img
        .as_raw()
        .iter()
        .map(|&pix| Complex::new(pix as f64 / 255.0, 0.0))
        .collect();

    img_buffer
}


fn transpose<T: Copy + Default>(width: usize, height: usize, matrix: &[T]) -> Vec<T> {
    let mut ind = 0;
    let mut ind_tr;
    let mut transposed = vec![T::default(); matrix.len()];
    for row in 0..height {
        ind_tr = row;
        for _ in 0..width {
            transposed[ind_tr] = matrix[ind];
            ind += 1;
            ind_tr += height;
        }
    }
    transposed
}




#[test]
fn test_image_fft(){
    let files = vec!["img/sjb-aerial.png", "img/mandrill.jpg"];

    for file in files {
        let img = image::open(file).unwrap().into_luma8();
        dynimg2complex(img);
    }
}


