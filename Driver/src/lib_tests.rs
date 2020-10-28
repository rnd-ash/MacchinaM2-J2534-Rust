// Contains all the tests for the J2534 library interior  functions from 'passthru_drv.rs'

#[cfg(test)]
mod tests {
    use crate::passthru_drv;
    use crate::comm::*;
    use J2534Common::*;

    #[test]
    fn test_open_close() {
        let mut dev: u32 = 0;
        assert!(passthru_drv::passthru_open(&mut dev) as i32 == PassthruError::STATUS_NOERROR as i32);
        assert!(dev == 0x1234);
        std::thread::sleep(std::time::Duration::from_millis(500)); 
        let send = COMM_MSG::new_with_args(MsgType::TestMessage, &[0x00]);
        match M2.write().as_deref_mut() {
            Ok(d) => {
                if let Some(dev) =d {
                    match dev.write_and_read(send, 100) {
                        Some(x) => assert!(x == send),
                        None => panic!("Did not receive response!")
                    }
                }
            },
            Err(x) => eprintln!("Error {:?}", x)
        };
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(passthru_drv::passthru_close(dev) as i32 == PassthruError::STATUS_NOERROR as i32);
    }

    #[test]
    fn test_batt() {
        let mut dev: u32 = 0;
        assert!(passthru_drv::passthru_open(&mut dev) as i32 == PassthruError::STATUS_NOERROR as i32);
        for _ in 0..10 {
            if let Some(vbatt) = get_batt_voltage() {
                println!("M2 Read voltage: {}V", vbatt);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        passthru_drv::passthru_close(dev);
    }
}