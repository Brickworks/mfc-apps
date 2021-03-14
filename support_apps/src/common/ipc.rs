pub static NNG_TX_ADDR: &str = "ipc:///tmp/nucleus";
pub static NNG_PWM_ADDR: &str = "ipc:///tmp/nucleus_pwm"; 

pub fn fmt_nng_msg(topic: &str, body: &[u8]) -> Vec<u8> {
    [topic.as_bytes(), ":".as_bytes(), body].concat()
}
