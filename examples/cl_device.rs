#[cfg(feature="opencl")]
use custos::{BaseOps, VecRead, Matrix, Error, CLDevice};

#[cfg(feature="opencl")]
fn main() -> Result<(), Error> {
    let device = CLDevice::get(0)?;
    
    let a = Matrix::<f32>::new(&device, (5, 5));
    let b = Matrix::from((&device, (5, 5), vec![1.3; 5*5]));

    let out = device.add(&a, &b);

    assert_eq!(device.read(out.data()), vec![1.3; 5*5]);
    Ok(())
}

#[cfg(not(feature="opencl"))]
fn main() {}