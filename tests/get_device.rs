use custos::{get_device, AsDev, Buffer, VecRead, CPU};

#[test]
fn get_device_test_cpu() {
    let device = CPU::new();

    let buf = Buffer::from((&device, [1., 1.5, 0.14]));

    let read_device = get_device!(device.dev(), VecRead<f32>);
    assert_eq!(vec![1., 1.5, 0.14], read_device.read(&buf));
}

#[cfg(feature = "opencl")]
#[test]
fn get_device_test_cl() -> custos::Result<()> {
    use custos::OpenCL;

    let device = OpenCL::new(0)?;

    let buf = Buffer::from((&device, [1., 1.5, 0.14]));

    let read_device = get_device!(device.dev(), VecRead<f32>);
    assert_eq!(vec![1., 1.5, 0.14], read_device.read(&buf));
    Ok(())
}

#[cfg(feature = "cuda")]
#[test]
fn get_device_test_cu() -> custos::Result<()> {
    use custos::CUDA;

    let device = CUDA::new(0)?;

    let buf = Buffer::from((&device, [1., 1.5, 0.14]));

    let read_device = get_device!(device.dev(), VecRead<f32>);
    assert_eq!(vec![1., 1.5, 0.14], read_device.read(&buf));
    Ok(())
}
