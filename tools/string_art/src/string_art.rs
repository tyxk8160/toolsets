use std::collections::HashMap;
use std::collections::HashSet;
use std::f64::consts::PI;

use std::sync::Arc;

use std::usize;

use image::{ DynamicImage, GenericImageView};

// use image::GenericImage;
use image::ImageBuffer;
// use image::{DynamicImage, GenericImageView};   draw_antialiased_line_segment_mut,
use imageproc::drawing::{
    draw_line_segment_mut, BresenhamLineIter,
};


use rayon::prelude::*;

// 为了代码通用性， 图片只需要实现以下trait
// get_pixel(x, y) -> pixel
// set_pixel(x, y)

#[derive(Debug)]
pub struct StringArt {
    pub input_image: DynamicImage, //输入图片信息
    pub block: u32,                // 图钉画块大小
    pub pin_nums: usize,           // 设置图钉数
    pub max_lines: usize,          // 设置最多的绕线数

    pub width: u32,  // 输出画的宽
    pub height: u32, // 输出画的高

    pub pin_image: DynamicImage, // 输出图钉画图片的尺寸

    pub lines: Vec<(usize, usize)>,      // 记录线条数
    pub pins: Vec<(u32, u32)>,           // 记录所有图钉的位置信息
    _cnt: usize,                         // 前已经画的线条数
    _draw_lins: HashSet<(usize, usize)>, // 记录已经画了的线
    _blocks: Vec<Vec<u32>>,              // 记录已经画的block
}

impl StringArt {
    fn from_image(
        origin: DynamicImage,
        block: Option<u32>,
        pin_nums: Option<usize>,
        max_lines: Option<usize>,
    ) -> Self {
        let _block = block.unwrap_or(16);
        let _pin_nums = pin_nums.unwrap_or(255);
        let _max_lines = max_lines.unwrap_or(4000);
        let block_width = origin.width();
        let block_height = origin.height();
        let width = origin.width() * _block;
        let height = origin.height() * _block;

        // todo: 采用纯白图初始化，可以将color作为一个参数传入

        let white_img =
            ImageBuffer::from_fn(width, height, |_x, _y| image::Rgba([255, 255, 255, 255]));
        let init_img = DynamicImage::ImageRgba8(white_img);
        let pins = Self::gen_pins(width, height, _pin_nums);

        Self {
            input_image: origin,
            block: _block,
            pin_nums: _pin_nums,
            max_lines: _max_lines,
            width,
            height,
            pin_image: init_img,
            pins,
            lines: Vec::new(),
            _cnt: 0,
            _draw_lins: HashSet::new(),
            _blocks: vec![vec![0; block_width as usize]; block_height as usize],
        }
    }

    pub fn gen_pins(width: u32, height: u32, pin_nums: usize) -> Vec<(u32, u32)> {
        let _width = width as f64;
        let _height = height as f64;
        let c_x = _width / 2.0;
        let c_y = _height / 2.0;
        let radius = c_x - 1.0; // 避免溢出
        let mut pins = vec![(0, 0); pin_nums];

        let pin_nums_f = pin_nums as f64;
        for i in 0..pin_nums {
            // 屏幕左上角(0,0)
            let x = c_x + (radius * (i as f64 * 2.0 * PI / pin_nums_f).cos()).round();
            let y = c_y - (radius * (i as f64 * 2.0 * PI / pin_nums_f).sin()).round();
            pins[i] = (x as u32, y as u32);
        }
        pins
    }

    
    fn _update_blocks(&mut self, start: usize, end: usize){

         // 计算变化的block数
         let p1 = self.pins[start];
         let p2 = self.pins[end];
 
         let bl = BresenhamLineIter::new((p1.0 as f32, p1.1 as f32), (p2.0 as f32, p2.1 as f32));
 
         // 计算所有变化的点
         let points: Vec<(i32, i32)> = bl
             .filter(|&x| {
                 self.pin_image.in_bounds(x.0 as u32, x.1 as u32)
                     && self.pin_image.get_pixel(x.0 as u32, x.1 as u32)[0] == 255
             })
             .collect();
    
         for &item in &points {
            let block_x = (item.0 as u32)/self.block ;
            let block_y = item.1 as u32 / self.block;
            self._blocks[block_y as usize][block_x as usize] += 1;
         }
 

    }

    pub fn get_line_score(&self, start: usize, end: usize) -> f64 {
        // 计算变化的block数
        let p1 = self.pins[start];
        let p2 = self.pins[end];

        let bl = BresenhamLineIter::new((p1.0 as f32, p1.1 as f32), (p2.0 as f32, p2.1 as f32));

        // 计算所有变化的点
        let points: Vec<(i32, i32)> = bl
            .filter(|&x| {
                self.pin_image.in_bounds(x.0 as u32, x.1 as u32)
                    && self.pin_image.get_pixel(x.0 as u32, x.1 as u32)[0] == 255
            })
            .collect();
        let mut block_counts: HashMap<(u32, u32), u32> = HashMap::new();
        for &item in &points {
            let block = (item.0 as u32 / self.block, item.1 as u32 / self.block);
            *block_counts.entry(block).or_insert(0) += 1
        }

        let mut score = 0 as f64;
        for (&block, &cnt) in &block_counts {
            let pixel = self._blocks[block.1 as usize][block.0 as usize];
            // let current = (pixel+cnt) as f64 / ((self.block *self.block) as f64 );
            let target = 1.0 - self.input_image.get_pixel(block.0, block.1)[0] as f64 / 255.0;
            // 计算得分变化
            // (target - pixel)^2 - (target-current)^2
            let tmp1 = 2.0 * target - (2 * pixel + cnt) as f64 / (self.block * self.block) as f64;
            score += (cnt as f64) * tmp1
        }
        let n = block_counts.len();

        if n > 0 {
            return score as f64;
        }
        return std::f64::NEG_INFINITY;
    }

    pub fn update_one_line(&mut self, current_pin: usize)->usize {
        let  end_pins: Vec<usize> = (0..self.pin_nums)
            .filter(|p| {
                current_pin != *p
                    && !(self._draw_lins.contains(&(current_pin, *p))
                        || self._draw_lins.contains(&(*p, current_pin)))
            })
            .collect();

        let max_score_item: Option<(usize, f64)>;

        {
            let self_ref = Arc::new(&self);
            max_score_item = end_pins
                .par_iter()
                .map(|end_| {
                    let end = *end_;
                    let self_get = self_ref.clone();

                    let score = self_get.get_line_score(current_pin, end);
                    (end, score)
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            // .max_by(|a: &(&usize, f64), b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        let mut next_pin = current_pin;
        if let Some((end_pos, score)) = max_score_item {
            if score > 0.0 {
                let (start_x, start_y) = self.pins[current_pin];
                let (end_x, end_y) = self.pins[end_pos];
                println!("draw:{:?}->{:?}", current_pin, end_pos);
                // 更新blocks结构
                self._update_blocks(current_pin, end_pos);

                draw_line_segment_mut(
                    &mut self.pin_image,
                    (start_x as f32, start_y as f32),
                    (end_x as f32, end_y as f32),
                    image::Rgba([0, 0, 0, 255]),
                );


                // // 画抗锯齿直线
                // draw_antialiased_line_segment_mut(
                //     &mut self.pin_image,
                //     (start_x as i32, start_y as i32),
                //     (end_x as i32, end_y as i32),
                //     image::Rgba([0, 0, 0, 160]),
                //     interpolate,
                // );

            
                self._draw_lins.insert((current_pin, end_pos));

                self._draw_lins.insert(( end_pos, current_pin));
                self.lines.push((current_pin, end_pos));
                self._cnt += 1;
                next_pin = end_pos;

            }
        }

        next_pin
    }
}


#[test]
fn test_img_load(){
    use image::io::Reader as ImageReader;
    use image::imageops::FilterType;
    
    let _input = ImageReader::open("C:\\Users\\tyxk8160\\Downloads\\ae300.jpg")
        .unwrap()
        .decode()
        .unwrap().grayscale();

       
    let input = _input.resize(256, 256, FilterType::Lanczos3);

    input.save("lc_png_origin.png").unwrap();

    let mut st: StringArt =StringArt::from_image(input, Some(16), Some(288), Some(4000));

    let mut current_pin = 2 as usize;

    let mut need_draw=false;

    for i in 1..10000 {
        let next_pin = st.update_one_line(current_pin);

        if next_pin == current_pin {
            println!("not update. current:{:?}", current_pin);
            current_pin = (current_pin+18)% st.pin_nums;
            if need_draw{
                let filename = format!("image-{}.png", i);
                st.pin_image.save(filename).unwrap();
                need_draw=false;

            }
            continue;
        }
        if !need_draw{
            need_draw=true;
        }
        current_pin = next_pin;

        if i % 500 == 0 {
            let filename = format!("image-{}.png", i);
            st.pin_image.save(filename).unwrap();
        }
    }

    st.pin_image.save("lc_png_test.png").unwrap();

     
}