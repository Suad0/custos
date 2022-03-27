use custos::{libs::{cpu::CPU, opencl::cl_device::CLDevice}, AsDev, Matrix, range};


#[test]
fn test_rc_get_dev() {
    
    {
        let device = CPU::new().select();
        let a = Matrix::from(( &device, (2, 3), [1., 2., 3., 4., 5., 6.,]));
        let b = Matrix::from(( &device, (2, 3), [6., 5., 4., 3., 2., 1.,]));

        for _ in range(100) {
            let c = a + b;
            assert_eq!(&[7., 7., 7., 7., 7., 7.,], c.as_cpu_slice());
        }
        
    }
    let device = CLDevice::get(0).unwrap().select();

    let a = Matrix::from(( &device, (2, 3), [1f32, 2., 3., 4., 5., 6.,]));
    let b = Matrix::from(( &device, (2, 3), [6., 5., 4., 3., 2., 1.,]));

    let c = a+b;
    println!("{:?}", c.read());
    
}