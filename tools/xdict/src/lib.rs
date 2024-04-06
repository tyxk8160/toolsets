
use std::time:: Instant;
use xcap::Monitor;


pub fn xcapture(){
    let start = Instant::now();
    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        let image = monitor.capture_image().unwrap();

        image
            .save(format!("monitor-{}.png", 1))
            .unwrap();
    }

    println!("运行耗时: {:?}", start.elapsed());

} 



#[test]
fn test_capture(){
    xcapture()
}