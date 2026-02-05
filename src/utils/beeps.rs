use actually_beep::beep_with_hz_and_millis;

pub fn beep_break() {
    let _ = beep_with_hz_and_millis(400, 200);
    let _ = beep_with_hz_and_millis(200, 200);
    let _ = beep_with_hz_and_millis(150, 200);
}
pub fn beep_work() {
    let _ = beep_with_hz_and_millis(400, 200);
    let _ = beep_with_hz_and_millis(500, 200);
    let _ = beep_with_hz_and_millis(400, 200);
    let _ = beep_with_hz_and_millis(600, 200);
}
pub fn beep_finish() {
    let _ = beep_with_hz_and_millis(300, 200);
    let _ = beep_with_hz_and_millis(400, 150);
    let _ = beep_with_hz_and_millis(350, 200);
    let _ = beep_with_hz_and_millis(450, 150);
    let _ = beep_with_hz_and_millis(400, 200);
    let _ = beep_with_hz_and_millis(200, 250);
}
