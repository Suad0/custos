use custos::{CPU, Buffer, opencl::{tew, api::{unified_mem, enqueue_map_buffer}}};
#[cfg(feature="opencl")]
use custos::{CLDevice, Error};

#[cfg(feature="opencl")]
#[test]
fn test_unified_mem_bool() -> Result<(), Error> {
    let device = CLDevice::get(0)?;
    let um = device.unified_mem()?;
    println!("um: {um}");
    Ok(())
}

#[cfg(feature="opencl")]
#[test]
fn test_unified_mem() -> Result<(), Error> {
    const TIMES: usize = 1000;
    use std::time::Instant;

    use custos::{opencl::api::{create_buffer, MemFlags, enqueue_map_buffer, release_mem_object}, Buffer};

    let len = 2000;

    let data = vec![1f32; len];

    let device = CLDevice::get(0)?;
    
    let before = Instant::now();
    for _ in 0..TIMES {
        //std::thread::sleep(std::time::Duration::from_secs(1));
        
        let buf = create_buffer(&device.ctx(), MemFlags::MemReadWrite | MemFlags::MemUseHostPtr, len, Some(&data))?;
        let ptr = unsafe { enqueue_map_buffer::<f32>(&device.queue(), buf, true, 2, 0, len)}? as *mut f32;
        let slice = unsafe {std::slice::from_raw_parts_mut(ptr, len)};
        
        for idx in 20..100 {
            slice[idx] = 4.;
        }

        unsafe { 
            release_mem_object(buf)?;
        }
        // 'data' vec is not freed
        assert_eq!(slice[25], 4.);
        /* 
        let mut read = vec![0f32; len];
        let event = unsafe { custos::opencl::api::enqueue_read_buffer(&device.queue(), buf, &mut read, true)}?;
        custos::opencl::api::wait_for_event(event)?;
        println!("read: {read:?}");
        */
    }
    let after = Instant::now();
    println!("use host ptr: {:?}", (after-before) / TIMES as u32);
        
    let before = Instant::now();
    for _ in 0..TIMES {        
        let buf = create_buffer(&device.ctx(), MemFlags::MemReadWrite | MemFlags::MemCopyHostPtr, len, Some(&data))?;
        let ptr = unsafe { enqueue_map_buffer::<f32>(&device.queue(), buf, true, 2, 0, len)}? as *mut f32;
        let slice = unsafe {std::slice::from_raw_parts_mut(ptr, len)};
        
        for idx in 20..100 {
            slice[idx] = 4.;
        }

        unsafe { 
            release_mem_object(buf)?;
        }
    }
    let after = Instant::now();
    println!("copy host ptr: {:?}", (after-before) / TIMES as u32);
    Ok(())
}

#[test]
fn test_unified_calc() -> Result<(), Error> {

    let len = 100;
    
    let device = CPU::new();
    let mut a = Buffer::<f32>::new(&device, len);
    let mut b = Buffer::<f32>::from((&device, vec![1.; len]));

    let cl = CLDevice::get(0)?;
    
    let a = Buffer {
        ptr: unified_mem(cl.ctx(), a.as_slice_mut())? as *mut f32,
        len
    };
    let b = Buffer {
        ptr: unified_mem(cl.ctx(), b.as_slice_mut())? as *mut f32,
        len,
    };
    

    Ok(())
}


