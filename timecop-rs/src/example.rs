
fn main() {
    let mut target = [0 as u8; 32];
    for i in 0..32 {
        target[i] = i as u8;
        timecop::poison(&target);

        let mut status = 0;

        if target[0] / 2 == 0 {
            status = 1;
        }

        timecop::unpoison(&target);
        println!("{}", status);
    }
}