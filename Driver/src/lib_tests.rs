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
                        Ok(x) => assert!(x == send),
                        Err(_) => panic!("Did not receive response!")
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

        let mut v: u32 = 0;
        let p_input = std::ptr::null_mut::<libc::c_void>();
        let p_output = (&mut v) as *mut _ as *mut libc::c_void;
        for _ in 0..10 {
            passthru_drv::passthru_ioctl(0, J2534Common::IoctlID::READ_VBATT as u32, p_input, p_output);
            println!("V: {}", v);
        }
        passthru_drv::passthru_close(dev);
    }

    #[test]
    fn test_connect() {
        let mut dev: u32 = 0;
        let mut channel_id: u32 = 0;
        assert!(passthru_drv::passthru_open(&mut dev) as i32 == PassthruError::STATUS_NOERROR as i32);

        passthru_drv::passthru_connect(dev, Protocol::ISO15765 as u32, 0x0000, 500000, &mut channel_id);

        passthru_drv::passthru_close(dev);
    }

    #[test]
    fn test_fw_failure() {
        let mut fw_version: [u8; 80] = [0; 80];
        let mut dll_version: [u8; 80] = [0; 80];
        let mut api_version: [u8; 80] = [0; 80];
        assert_eq!(passthru_drv::passthru_read_version(fw_version.as_mut_ptr() as *mut i8, dll_version.as_mut_ptr() as *mut i8, api_version.as_mut_ptr() as *mut i8), PassthruError::ERR_DEVICE_NOT_CONNECTED);
    }

    #[test]
    fn test_fw_ok() {
        let mut dev: u32 = 0;
        let mut fw_version: [u8; 80] = [0; 80];
        let mut dll_version: [u8; 80] = [0; 80];
        let mut api_version: [u8; 80] = [0; 80];
        assert_eq!(passthru_drv::passthru_open(&mut dev), PassthruError::STATUS_NOERROR);
        assert_eq!(passthru_drv::passthru_read_version(fw_version.as_mut_ptr() as *mut i8, dll_version.as_mut_ptr() as *mut i8, api_version.as_mut_ptr() as *mut i8), PassthruError::STATUS_NOERROR);
        println!("FW VERSION {}", String::from_utf8(fw_version.to_vec()).unwrap());
        println!("API VERSION {}", String::from_utf8(api_version.to_vec()).unwrap());
        println!("DLL VERSION {}", String::from_utf8(dll_version.to_vec()).unwrap());
    }
}