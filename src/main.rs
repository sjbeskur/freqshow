use std::io::Cursor;
use image::{io::Reader as ImageReader, DynamicImage};

use rustfft::{FftPlanner, num_complex::Complex, FftDirection};
use show_image::{ImageView, ImageInfo, create_window};


fn main() -> Result<(), Box<dyn std::error::Error>> {    

    let args: Vec<_> = std::env::args().collect();
    if args.len() < 1 {
        println!("usage: freqshow <image_file_name>");
    }
    let image = read_image( args[1].clone()); //mandrill.jpg

    let (width , height) = image.dimensions();
    let mut img_buffer = dynimg2complex(image);
    fft_forward(width as usize, height as usize, &mut img_buffer);


    // Convert the complex img_buffer back into a gray image.
    let img_raw: Vec<u8> = img_buffer
        .iter()
        .map(|c| (c.norm().min(1.0) * 255.0) as u8)
        .collect();

    let out_img = image::GrayImage::from_raw(width, height, img_raw).unwrap();

    out_img.save(format!("output/fftlp-{}",args[0]));
    

    // Create a window with default options and display the image.
//    let window = create_window("image", Default::default())?;
//    window.set_image("FFT Lowpass filtered image (i think)", out_img).unwrap();
    Ok(())


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


